// Copyright 2019 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

use super::Error;
use futures::Future;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Mailbody {
    #[serde(deserialize_with = "super::protocol::deserialize_format")]
    _format: (),
    #[serde(deserialize_with = "super::protocol::deserialize_base64")]
    pub text: Vec<u8>,
}

pub fn fetch_mailbody<C: 'static + hyper::client::connect::Connect>(
    client: &hyper::Client<C, hyper::Body>,
    access_token: &str,
    body: &str,
) -> impl hyper::rt::Future<Error = Error, Item = Vec<u8>> {
    let url = format!("https://mail.tutanota.com/rest/tutanota/mailbody/{}", body);
    super::authenticated_get::get(client, access_token, &url).and_then(|response_body| {
        match serde_json::from_slice::<Mailbody>(&response_body) {
            Err(error) => Err(Error::Format(error)),
            Ok(response_data) => Ok(response_data.text),
        }
    })
}
