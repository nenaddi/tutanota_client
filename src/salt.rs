// Copyright 2019 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

use super::Error;
use futures::{
    future::{self, Either},
    Future, Stream,
};
use serde_derive::Deserialize;

#[derive(Deserialize)]
struct Response {
    #[serde(deserialize_with = "super::protocol::deserialize_format")]
    _format: (),
    #[serde(deserialize_with = "super::protocol::deserialize_base64")]
    salt: Vec<u8>,
}

pub fn fetch_salt<C: 'static + hyper::client::connect::Connect>(
    client: &hyper::Client<C, hyper::Body>,
    email_address: &str,
) -> impl hyper::rt::Future<Error = Error, Item = Vec<u8>> {
    let email_address = serde_json::to_string(email_address).unwrap();
    let email_address = urlencoding::encode(&email_address);
    let url = format!(
        "https://mail.tutanota.com/rest/sys/saltservice?_body=%7B%22_format%22%3A%220%22%2C%22mailAddress%22%3A{}%7D",
        email_address
    );
    client
        .get(url.parse().unwrap())
        .then(|result| match result {
            Err(error) => Either::A(future::err(Error::Network(error))),
            Ok(response) => {
                if response.status() != hyper::StatusCode::OK {
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
                            match serde_json::from_slice::<Response>(&response_body) {
                                Err(error) => Err(Error::Format(error)),
                                Ok(response_data) => Ok(response_data.salt),
                            }
                        }
                    }))
                }
            }
        })
}
