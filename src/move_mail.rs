// Copyright 2019 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

use super::Error;
use futures::Future;
use serde_derive::Serialize;

#[derive(Serialize)]
struct Request<'a> {
    #[serde(with = "super::protocol::format")]
    _format: (),
    mails: &'a [&'a (String, String)],
    #[serde(rename = "targetFolder")]
    target_folder: &'a (String, String),
}

pub fn move_mail<C: 'static + hyper::client::connect::Connect>(
    client: &hyper::Client<C, hyper::Body>,
    access_token: &str,
    mails: &[&(String, String)],
    target_folder: &(String, String),
) -> impl futures::Future<Error = Error, Item = ()> {
    let request_body = serde_json::to_string(&Request {
        _format: (),
        mails,
        target_folder,
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
        hyper::Uri::from_static("https://mail.tutanota.com/rest/tutanota/movemailservice");
    client.request(request).then(|result| match result {
        Err(error) => Err(Error::Network(error)),
        Ok(response) => {
            if response.status() == hyper::StatusCode::CREATED {
                Ok(())
            } else {
                Err(Error::Status(response))
            }
        }
    })
}
