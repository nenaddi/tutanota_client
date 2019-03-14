// Copyright 2019 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

use super::Error;
use futures::Future;
use serde_derive::Deserialize;

#[derive(Deserialize)]
struct Response {
    #[serde(with = "super::protocol::format")]
    _format: (),
    #[serde(rename = "systemFolders")]
    system_folders: SystemFolders,
}

#[derive(Deserialize)]
struct SystemFolders {
    folders: String,
}

pub fn fetch_mailbox<C: 'static + hyper::client::connect::Connect>(
    client: &hyper::Client<C, hyper::Body>,
    access_token: &str,
    mailbox: &str,
) -> impl hyper::rt::Future<Error = Error, Item = String> {
    let url = format!(
        "https://mail.tutanota.com/rest/tutanota/mailbox/{}",
        mailbox
    );
    super::authenticated_get::get(client, access_token, &url).and_then(|response_body| {
        match serde_json::from_slice::<Response>(&response_body) {
            Err(error) => Err(Error::Format(error)),
            Ok(response_data) => Ok(response_data.system_folders.folders),
        }
    })
}
