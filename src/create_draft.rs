// Copyright 2019 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

use super::Error;
use futures::{
    future::{self, Either},
    Future, Stream,
};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftData<'a> {
    // XXX What's the proper type?
    pub added_attachments: &'a [()],
    pub bcc_recipients: &'a [Recipient<'a>],
    #[serde(with = "super::protocol::base64")]
    pub body_text: Vec<u8>,
    pub cc_recipients: &'a [Recipient<'a>],
    #[serde(with = "super::protocol::base64")]
    pub confidential: Vec<u8>,
    #[serde(rename = "_id")]
    pub id: &'a str,
    // XXX What's the proper type?
    pub removed_attachments: &'a [()],
    // XXX What's the proper type?
    pub reply_tos: &'a [()],
    pub sender_mail_address: &'a str,
    #[serde(with = "super::protocol::base64")]
    pub sender_name: Vec<u8>,
    #[serde(with = "super::protocol::base64")]
    pub subject: Vec<u8>,
    pub to_recipients: &'a [Recipient<'a>],
}

#[derive(Serialize)]
pub struct Recipient<'a> {
    #[serde(rename = "_id")]
    pub id: &'a str,
    #[serde(rename = "mailAddress")]
    pub mail_address: &'a str,
    #[serde(with = "super::protocol::base64")]
    pub name: Vec<u8>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Request<'a> {
    // XXX What's the proper type?
    #[serde(with = "super::protocol::format")]
    conversation_type: (),
    #[serde(rename = "_format", with = "super::protocol::format")]
    format: (),
    draft_data: DraftData<'a>,
    owner_enc_session_key: String,
    // XXX What's the proper type?
    previous_message_id: (),
    sym_enc_session_key: String,
}

#[derive(Deserialize)]
struct Response {
    #[serde(with = "super::protocol::format")]
    _format: (),
    draft: (String, String),
}

pub fn create_draft<C: 'static + hyper::client::connect::Connect>(
    client: &hyper::Client<C, hyper::Body>,
    access_token: &str,
    session_key: [u8; 16],
    mail_group_key: [u8; 16],
    user_group_key: [u8; 16],
    draft_data: DraftData,
) -> impl futures::Future<Error = Error, Item = (String, String)> {
    let request_body = serde_json::to_string(&Request {
        conversation_type: (),
        format: (),
        draft_data,
        owner_enc_session_key: base64::encode(&super::encrypt_key(mail_group_key, session_key)[..]),
        previous_message_id: (),
        sym_enc_session_key: base64::encode(&super::encrypt_key(user_group_key, session_key)[..]),
    })
    .unwrap();
    let mut request = hyper::Request::new(hyper::Body::from(request_body));
    *request.method_mut() = hyper::Method::POST;
    // XXX Don't unwrap, but gracefully return error.
    request.headers_mut().insert(
        "accessToken",
        hyper::header::HeaderValue::from_str(access_token).unwrap(),
    );
    *request.uri_mut() =
        hyper::Uri::from_static("https://mail.tutanota.com/rest/tutanota/draftservice");
    client.request(request).then(|result| match result {
        Err(error) => Either::A(future::err(Error::Network(error))),
        Ok(response) => {
            if response.status() != hyper::StatusCode::CREATED {
                Either::A(future::err(Error::Status(response)))
            } else if match response.headers().get(hyper::header::CONTENT_TYPE) {
                None => true,
                Some(value) => value.as_bytes() != b"application/json;charset=utf-8",
            } {
                Either::A(future::err(Error::ContentType(response)))
            } else {
                Either::B(response.into_body().concat2().then(|result| match result {
                    Err(error) => Err(Error::Network(error)),
                    Ok(response_body) => match serde_json::from_slice::<Response>(&response_body) {
                        Err(error) => Err(Error::Format(error)),
                        Ok(response_data) => Ok(response_data.draft),
                    },
                }))
            }
        }
    })
}
