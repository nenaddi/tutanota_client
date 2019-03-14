// Copyright 2019 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

use super::Error;
use futures::Future;
use serde_derive::Serialize;

#[derive(Serialize)]
struct Request<'a> {
    folders: &'a [&'a (String, String)],
    #[serde(rename = "_format")]
    format: &'a str,
}

pub fn delete_mail_folder<C: 'static + hyper::client::connect::Connect>(
    client: &hyper::Client<C, hyper::Body>,
    access_token: &str,
    folder: &(String, String),
) -> impl futures::Future<Error = Error, Item = ()> {
    let request_body = serde_json::to_string(&Request {
        folders: &[folder],
        format: "0",
    })
    .unwrap();
    let mut request = hyper::Request::new(hyper::Body::from(request_body));
    *request.method_mut() = hyper::Method::DELETE;
    // XXX Don't unwrap, but gracefully return error.
    request.headers_mut().insert(
        "accessToken",
        hyper::header::HeaderValue::from_str(access_token).unwrap(),
    );
    request
        .headers_mut()
        .insert("v", hyper::header::HeaderValue::from_str("30").unwrap());
    *request.uri_mut() =
        hyper::Uri::from_static("https://mail.tutanota.com/rest/tutanota/mailfolderservice");
    client.request(request).then(|result| match result {
        Err(error) => Err(Error::Network(error)),
        Ok(response) => {
            if response.status() == hyper::StatusCode::OK {
                Ok(())
            } else {
                Err(Error::Status(response))
            }
        }
    })
}
