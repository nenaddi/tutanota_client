// Copyright 2019 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

use super::Error;
use futures::Future;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Mail {
    #[serde(with = "super::protocol::format")]
    _format: (),
    #[serde(rename = "_area")]
    pub area: String,
    pub attachments: Vec<(String, String)>,
    pub body: String,
    // XXX What's the proper type?
    #[serde(rename = "bccRecipients")]
    pub bcc_recipients: Vec<()>,
    // XXX What's the proper type?
    #[serde(rename = "ccRecipients")]
    pub cc_recipients: Vec<()>,
    #[serde(with = "super::protocol::base64")]
    pub confidential: Vec<u8>,
    #[serde(rename = "conversationEntry")]
    pub conversation_entry: (String, String),
    // XXX What's the proper type?
    #[serde(rename = "differentEnvelopeSender")]
    pub different_envelope_sender: (),
    // XXX What's the proper type?
    pub headers: (),
    #[serde(rename = "_id")]
    pub id: (String, String),
    // XXX What's the proper type?
    #[serde(rename = "listUnsubscribe")]
    pub list_unsubscribe: String,
    #[serde(rename = "movedTime")]
    pub moved_time: String,
    #[serde(rename = "_owner")]
    pub owner: String,
    #[serde(with = "super::protocol::base64", rename = "_ownerEncSessionKey")]
    pub owner_enc_session_key: Vec<u8>,
    #[serde(rename = "_ownerGroup")]
    pub owner_group: String,
    #[serde(rename = "_permissions")]
    pub permissions: String,
    #[serde(rename = "receivedDate")]
    pub received_date: String,
    // XXX What's the proper type?
    #[serde(rename = "replyTos")]
    pub reply_tos: Vec<()>,
    // XXX What's the proper type?
    #[serde(rename = "replyType")]
    pub reply_type: String,
    // XXX What's the proper type?
    pub restrictions: (),
    #[serde(rename = "sentDate")]
    pub sent_date: String,
    pub sender: Sender,
    pub state: String,
    #[serde(with = "super::protocol::base64")]
    pub subject: Vec<u8>,
    #[serde(rename = "toRecipients")]
    pub to_recipients: Vec<Sender>,
    pub trashed: String,
    pub unread: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Sender {
    pub address: String,
    // XXX What's the proper type?
    pub contact: (),
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(with = "super::protocol::base64")]
    pub name: Vec<u8>,
}

pub fn fetch_mail<C: 'static + hyper::client::connect::Connect>(
    client: &hyper::Client<C, hyper::Body>,
    access_token: &str,
    mails: &str,
) -> impl futures::Future<Error = Error, Item = Vec<Mail>> {
    let url = format!(
        "https://mail.tutanota.com/rest/tutanota/mail/{}?start=zzzzzzzzzzzz&count=100&reverse=true",
        mails
    );
    super::authenticated_get::get(client, access_token, &url).and_then(|response_body| {
        serde_json::from_slice::<Vec<Mail>>(&response_body).map_err(Error::Format)
    })
}
