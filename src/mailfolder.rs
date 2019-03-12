// Copyright 2019 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

use super::Error;
use futures::Future;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Folder {
    pub mails: String,
    #[serde(deserialize_with = "super::protocol::deserialize_format")]
    _format: (),
}

pub fn fetch_mailfolder<C: 'static + hyper::client::connect::Connect>(
    client: &hyper::Client<C, hyper::Body>,
    access_token: &str,
    folders: &str,
) -> impl hyper::rt::Future<Error = Error, Item = Vec<Folder>> {
    let url = format!("https://mail.tutanota.com/rest/tutanota/mailfolder/{}?start=------------&count=1000&reverse=false", folders);
    super::authenticated_get::get(client, access_token, &url).and_then(|response_body| {
        serde_json::from_slice::<Vec<Folder>>(&response_body).map_err(Error::Format)
    })
}
