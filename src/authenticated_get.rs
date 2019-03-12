// Copyright 2019 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

use super::Error;
use futures::{
    future::{self, Either},
    Future, Stream,
};

pub fn get<C: 'static + hyper::client::connect::Connect>(
    client: &hyper::Client<C, hyper::Body>,
    access_token: &str,
    url: &str,
) -> impl hyper::rt::Future<Error = Error, Item = hyper::Chunk> {
    let mut request = hyper::Request::new(Default::default());
    // XXX Don't unwrap, but gracefully return error.
    request.headers_mut().insert(
        "accessToken",
        hyper::header::HeaderValue::from_str(access_token).unwrap(),
    );
    // XXX Don't unwrap, but gracefully return error.
    *request.uri_mut() = url.parse().unwrap();
    client.request(request).then(|result| match result {
        Err(error) => Either::A(future::err(Error::Network(error))),
        Ok(response) => {
            if response.status() != hyper::StatusCode::OK {
                Either::A(future::err(Error::Status(response)))
            } else if match response.headers().get(hyper::header::CONTENT_TYPE) {
                None => true,
                Some(value) => value.as_bytes() != b"application/json;charset=utf-8",
            } {
                Either::A(future::err(Error::ContentType(response)))
            } else {
                Either::B(response.into_body().concat2().map_err(Error::Network))
            }
        }
    })
}
