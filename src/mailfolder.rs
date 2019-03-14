// Copyright 2019 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

use super::Error;
use futures::Future;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Folder {
    #[serde(rename = "folderType")]
    pub folder_type: String,
    #[serde(with = "super::protocol::format")]
    _format: (),
    #[serde(rename = "_id")]
    pub id: (String, String),
    pub mails: String,
    #[serde(with = "super::protocol::base64")]
    pub name: Vec<u8>,
    #[serde(with = "super::protocol::base64", rename = "_ownerEncSessionKey")]
    pub owner_enc_session_key: Vec<u8>,
    #[serde(rename = "_ownerGroup")]
    pub owner_group: String,
    #[serde(rename = "parentFolder")]
    pub parent_folder: Option<(String, String)>,
    #[serde(rename = "_permissions")]
    pub permissions: String,
    #[serde(rename = "subFolders")]
    pub sub_folders: String,
}

pub fn fetch_mailfolder<C: 'static + hyper::client::connect::Connect>(
    client: &hyper::Client<C, hyper::Body>,
    access_token: &str,
    folders: &str,
) -> impl futures::Future<Error = Error, Item = Vec<Folder>> {
    let url = format!("https://mail.tutanota.com/rest/tutanota/mailfolder/{}?start=------------&count=1000&reverse=false", folders);
    super::authenticated_get::get(client, access_token, &url).and_then(|response_body| {
        serde_json::from_slice::<Vec<Folder>>(&response_body).map_err(Error::Format)
    })
}
