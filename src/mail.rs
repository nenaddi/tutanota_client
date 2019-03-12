// Copyright 2019 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

use super::Error;
use futures::Future;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Mail {
    #[serde(deserialize_with = "super::protocol::deserialize_format")]
    _format: (),
    pub attachments: Vec<(String, String)>,
    pub body: String,
    #[serde(
        deserialize_with = "super::protocol::deserialize_base64",
        rename = "_ownerEncSessionKey"
    )]
    pub owner_enc_session_key: Vec<u8>,
    pub sender: Sender,
    #[serde(deserialize_with = "super::protocol::deserialize_base64")]
    pub subject: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub struct Sender {
    pub address: String,
}

pub fn fetch_mail<C: 'static + hyper::client::connect::Connect>(
    client: &hyper::Client<C, hyper::Body>,
    access_token: &str,
    mails: &str,
) -> impl hyper::rt::Future<Error = Error, Item = Vec<Mail>> {
    let url = format!(
        "https://mail.tutanota.com/rest/tutanota/mail/{}?start=zzzzzzzzzzzz&count=100&reverse=true",
        mails
    );
    super::authenticated_get::get(client, access_token, &url).and_then(|response_body| {
        serde_json::from_slice::<Vec<Mail>>(&response_body).map_err(Error::Format)
    })
}
