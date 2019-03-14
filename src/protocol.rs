// Copyright 2019 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

/// A failure to make an HTTP request and parse the response.
///
/// Some variants contain the failing `hyper::Response` so it can be inspected.
#[derive(Debug)]
pub enum Error {
    /// The content type of the response was not recognized.
    ContentType(hyper::Response<hyper::Body>),
    /// The format of the response body was not recognized.
    Format(serde_json::Error),
    /// The HTTP request failed.
    Network(hyper::Error),
    /// The status code of the response was not recognized.
    Status(hyper::Response<hyper::Body>),
}

pub mod base64 {
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Vec<u8>, D::Error> {
        deserializer.deserialize_str(Base64Visitor)
    }

    struct Base64Visitor;

    impl<'de> serde::de::Visitor<'de> for Base64Visitor {
        type Value = Vec<u8>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "base64 string")
        }

        fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
            base64::decode(value).map_err(|_| {
                serde::de::Error::invalid_value(serde::de::Unexpected::Str(value), &self)
            })
        }
    }

    pub fn serialize<S: serde::Serializer>(
        value: &Vec<u8>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&base64::encode(value))
    }
}

pub mod format {
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<(), D::Error> {
        deserializer.deserialize_str(FormatVisitor)
    }

    struct FormatVisitor;

    impl<'de> serde::de::Visitor<'de> for FormatVisitor {
        type Value = ();

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "string \"0\"")
        }

        fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
            if value == "0" {
                Ok(())
            } else {
                Err(serde::de::Error::invalid_value(
                    serde::de::Unexpected::Str(value),
                    &self,
                ))
            }
        }
    }

    pub fn serialize<S: serde::Serializer>(_value: &(), serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str("0")
    }
}
