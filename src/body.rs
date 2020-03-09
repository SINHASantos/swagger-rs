/// Helper methods to act on hyper::Body

use std::pin::Pin;
use std::future::Future;
use futures::stream::{Stream, StreamExt};
use hyper::body::Bytes;

/// Additional function for hyper::Body
pub trait BodyExt {

    /// Raw body type
    type Raw;

    /// Error if we can't gather up the raw body
    type Error;


    /// Collect the body into a raw form
    fn to_raw(self) -> Pin<Box<dyn Future<Output=Result<Self::Raw, Self::Error>> + Send>>;
}

impl<T, E> BodyExt for T where
    T: Stream<Item=Result<Bytes, E>> + Unpin + Send + 'static
{
    type Raw = Vec<u8>;
    type Error = E;

    fn to_raw(mut self) -> Pin<Box<dyn Future<Output=Result<Self::Raw, Self::Error>> + Send>> {
        Box::pin(async {
            let mut raw = Vec::new();
            while let (Some(chunk), rest) = self.into_future().await {
                raw.extend_from_slice(&chunk?);
                self = rest;
            }
            Ok(raw)
        })
    }
}