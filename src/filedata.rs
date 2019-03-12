// Copyright 2019 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

use super::Error;
use futures::Future;

pub fn fetch_filedata<C: 'static + hyper::client::connect::Connect>(
    client: &hyper::Client<C, hyper::Body>,
    access_token: &str,
    file: &(String, String),
) -> impl hyper::rt::Future<Error = Error, Item = hyper::Body> {
    let mut request = hyper::Request::new(Default::default());
    // XXX Don't unwrap, but gracefully return error.
    request.headers_mut().insert(
        "accessToken",
        hyper::header::HeaderValue::from_str(access_token).unwrap(),
    );
    let url = format!(
        "https://mail.tutanota.com/rest/tutanota/filedataservice?_body=%7B%22_format%22%3A%220%22%2C%22base64%22%3A%220%22%2C%22file%22%3A%5B{}%2C{}%5D%7D",
        urlencoding::encode(&serde_json::to_string(&file.0).unwrap()),
        urlencoding::encode(&serde_json::to_string(&file.1).unwrap()),
    );
    // XXX Don't unwrap, but gracefully return error.
    *request.uri_mut() = url.parse().unwrap();
    client.request(request).then(|result| match result {
        Err(error) => Err(Error::Network(error)),
        Ok(response) => {
            if response.status() != hyper::StatusCode::OK {
                Err(Error::Status(response))
            } else if match response.headers().get(hyper::header::CONTENT_TYPE) {
                None => true,
                Some(value) => value.as_bytes() != b"application/octet-stream",
            } {
                Err(Error::ContentType(response))
            } else {
                Ok(response.into_body())
            }
        }
    })
}
