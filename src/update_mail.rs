// Copyright 2019 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

use super::Error;
use futures::Future;

pub fn update_mail<C: 'static + hyper::client::connect::Connect>(
    client: &hyper::Client<C, hyper::Body>,
    access_token: &str,
    mail: &super::mail::Mail,
) -> impl futures::Future<Error = Error, Item = ()> {
    let request_body = serde_json::to_string(&mail).unwrap();
    let mut request = hyper::Request::new(hyper::Body::from(request_body));
    *request.method_mut() = hyper::Method::PUT;
    // XXX Don't unwrap, but gracefully return error.
    request.headers_mut().insert(
        "accessToken",
        hyper::header::HeaderValue::from_str(access_token).unwrap(),
    );
    let url = format!(
        "https://mail.tutanota.com/rest/tutanota/mail/{}/{}",
        mail.id.0, mail.id.1
    );
    // XXX Don't unwrap, but gracefully return error.
    *request.uri_mut() = url.parse().unwrap();
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
