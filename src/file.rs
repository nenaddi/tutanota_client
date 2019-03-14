// Copyright 2019 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

use super::Error;
use futures::Future;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct File {
    pub data: String,
    #[serde(with = "super::protocol::format")]
    _format: (),
    #[serde(with = "super::protocol::base64", rename = "mimeType")]
    pub mime_type: Vec<u8>,
    #[serde(with = "super::protocol::base64")]
    pub name: Vec<u8>,
    #[serde(with = "super::protocol::base64", rename = "_ownerEncSessionKey")]
    pub owner_enc_session_key: Vec<u8>,
    pub size: String,
}

pub fn fetch_file<C: 'static + hyper::client::connect::Connect>(
    client: &hyper::Client<C, hyper::Body>,
    access_token: &str,
    file: &(String, String),
) -> impl futures::Future<Error = Error, Item = File> {
    let url = format!(
        "https://mail.tutanota.com/rest/tutanota/file/{}/{}",
        file.0, file.1
    );
    super::authenticated_get::get(client, access_token, &url).and_then(|response_body| {
        serde_json::from_slice::<File>(&response_body).map_err(Error::Format)
    })
}
