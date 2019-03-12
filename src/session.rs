// Copyright 2019 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

use super::Error;
use futures::{
    future::{self, Either},
    Future, Stream,
};
use serde_derive::{Deserialize, Serialize};
use sha2::Digest;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Request<'a> {
    access_key: (),
    auth_token: (),
    auth_verifier: String,
    client_identifier: &'a str,
    #[serde(rename = "_format")]
    format: &'a str,
    mail_address: &'a str,
    recover_code_verifier: (),
    user: (),
}

#[derive(Debug, Deserialize)]
pub struct Response {
    #[serde(deserialize_with = "super::protocol::deserialize_format")]
    _format: (),
    #[serde(rename = "accessToken")]
    pub access_token: String,
    pub user: String,
}

pub fn fetch_session<C: 'static + hyper::client::connect::Connect>(
    client: &hyper::Client<C, hyper::Body>,
    client_identifier: &str,
    email_address: &str,
    user_passphrase_key: &[u8],
) -> impl hyper::rt::Future<Error = Error, Item = Response> {
    let mut hasher = sha2::Sha256::new();
    hasher.input(user_passphrase_key);
    let hash = hasher.result();
    let auth_verifier = base64::encode_config(&hash, base64::URL_SAFE_NO_PAD);
    let request_body = serde_json::to_string(&Request {
        access_key: (),
        auth_token: (),
        auth_verifier,
        client_identifier,
        format: "0",
        mail_address: email_address,
        recover_code_verifier: (),
        user: (),
    })
    .unwrap();
    let mut request = hyper::Request::new(hyper::Body::from(request_body));
    *request.method_mut() = hyper::Method::POST;
    *request.uri_mut() =
        hyper::Uri::from_static("https://mail.tutanota.com/rest/sys/sessionservice");
    client.request(request).then(|result| match result {
        Err(error) => Either::A(future::err(Error::Network(error))),
        Ok(response) => {
            if response.status() != hyper::StatusCode::CREATED {
                Either::A(future::err(Error::Status(response)))
            } else if match response.headers().get(hyper::header::CONTENT_TYPE) {
                None => true,
                Some(value) => value.as_bytes() != b"application/json;charset=utf-8",
            } {
                Either::A(future::err(Error::ContentType(response)))
            } else {
                Either::B(response.into_body().concat2().then(|result| match result {
                    Err(error) => Err(Error::Network(error)),
                    Ok(response_body) => {
                        serde_json::from_slice::<Response>(&response_body).map_err(Error::Format)
                    }
                }))
            }
        }
    })
}
