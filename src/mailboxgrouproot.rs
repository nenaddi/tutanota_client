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
    mailbox: String,
}

pub fn fetch_mailboxgrouproot<C: 'static + hyper::client::connect::Connect>(
    client: &hyper::Client<C, hyper::Body>,
    access_token: &str,
    group: &str,
) -> impl futures::Future<Error = Error, Item = String> {
    let url = format!(
        "https://mail.tutanota.com/rest/tutanota/mailboxgrouproot/{}",
        group
    );
    super::authenticated_get::get(client, access_token, &url).and_then(|response_body| {
        match serde_json::from_slice::<Response>(&response_body) {
            Err(error) => Err(Error::Format(error)),
            Ok(response_data) => Ok(response_data.mailbox),
        }
    })
}
