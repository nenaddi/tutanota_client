// Copyright 2019 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

use super::Error;
use futures::Future;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Membership {
    pub group: String,
    pub group_type: String,
    #[serde(with = "super::protocol::base64")]
    pub sym_enc_g_key: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub struct Response {
    #[serde(with = "super::protocol::format")]
    _format: (),
    pub memberships: Vec<Membership>,
    #[serde(rename = "userGroup")]
    pub user_group: UserGroup,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserGroup {
    #[serde(with = "super::protocol::base64")]
    pub sym_enc_g_key: Vec<u8>,
}

pub fn fetch_user<C: 'static + hyper::client::connect::Connect>(
    client: &hyper::Client<C, hyper::Body>,
    access_token: &str,
    user: &str,
) -> impl futures::Future<Error = Error, Item = Response> {
    let url = format!("https://mail.tutanota.com/rest/sys/user/{}", user);
    super::authenticated_get::get(client, access_token, &url).and_then(|response_body| {
        serde_json::from_slice::<Response>(&response_body).map_err(Error::Format)
    })
}
