// Copyright 2019 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

use super::Error;
use futures::{
    future::{self, Either},
    Future, Stream,
};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Request<'a> {
    folder_name: String,
    #[serde(rename = "_format")]
    format: &'a str,
    owner_enc_session_key: String,
    parent_folder: &'a (String, String),
}

#[derive(Deserialize)]
struct Response {
    #[serde(with = "super::protocol::format")]
    _format: (),
    #[serde(rename = "newFolder")]
    new_folder: (String, String),
}

pub fn create_mail_folder<C: 'static + hyper::client::connect::Connect>(
    client: &hyper::Client<C, hyper::Body>,
    access_token: &str,
    group_key: [u8; 16],
    session_key: [u8; 16],
    parent_folder: &(String, String),
    name: &str,
) -> impl futures::Future<Error = Error, Item = String> {
    let request_body = serde_json::to_string(&Request {
        folder_name: base64::encode(&super::encrypt_with_mac(
            &super::SubKeys::new(session_key),
            name.as_bytes(),
        )),
        format: "0",
        owner_enc_session_key: base64::encode(&super::encrypt_key(group_key, session_key)),
        parent_folder,
    })
    .unwrap();
    let mut request = hyper::Request::new(hyper::Body::from(request_body));
    *request.method_mut() = hyper::Method::POST;
    // XXX Don't unwrap, but gracefully return error.
    request.headers_mut().insert(
        "accessToken",
        hyper::header::HeaderValue::from_str(access_token).unwrap(),
    );
    *request.uri_mut() =
        hyper::Uri::from_static("https://mail.tutanota.com/rest/tutanota/mailfolderservice");
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
                    Ok(response_body) => match serde_json::from_slice::<Response>(&response_body) {
                        Err(error) => Err(Error::Format(error)),
                        Ok(response_data) => Ok(response_data.new_folder.0),
                    },
                }))
            }
        }
    })
}
