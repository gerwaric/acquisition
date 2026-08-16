//! The single bounded wire hand-off seam shared by the actor and mock.

use std::future::Future;

use http::{HeaderMap, Request, Response, StatusCode};

use crate::header::MAX_HEADER_VALUE_BYTES;

pub type WireRequest = Request<Vec<u8>>;

/// The response contract is enforced at the transport boundary. A future
/// production client must cap its body read before allocating this `Vec`; no
/// raw `Response<Vec<u8>>` can be returned to the actor by the trait.
pub const MAX_RESPONSE_HEADERS: usize = 32;
pub const MAX_RESPONSE_HEADER_NAME_BYTES: usize = 256;
pub const MAX_RESPONSE_BODY_BYTES: usize = 16 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseBoundsError {
    TooManyHeaders { limit: usize },
    HeaderNameTooLong { limit: usize },
    HeaderValueTooLong { limit: usize },
    BodyTooLong { limit: usize },
}

/// A response proven bounded before it can cross the transport/actor seam.
#[derive(Debug)]
pub struct WireResponse(Response<Vec<u8>>);

impl WireResponse {
    pub fn try_new(response: Response<Vec<u8>>) -> Result<Self, ResponseBoundsError> {
        validate_response(&response)?;
        Ok(Self(response))
    }

    pub fn status(&self) -> StatusCode {
        self.0.status()
    }

    pub fn headers(&self) -> &HeaderMap {
        self.0.headers()
    }

    pub fn body(&self) -> &[u8] {
        self.0.body()
    }
}

fn validate_response(response: &Response<Vec<u8>>) -> Result<(), ResponseBoundsError> {
    if response.body().len() > MAX_RESPONSE_BODY_BYTES {
        return Err(ResponseBoundsError::BodyTooLong {
            limit: MAX_RESPONSE_BODY_BYTES,
        });
    }
    let headers = response.headers();
    if headers.len() > MAX_RESPONSE_HEADERS {
        return Err(ResponseBoundsError::TooManyHeaders {
            limit: MAX_RESPONSE_HEADERS,
        });
    }
    if headers
        .keys()
        .any(|name| name.as_str().len() > MAX_RESPONSE_HEADER_NAME_BYTES)
    {
        return Err(ResponseBoundsError::HeaderNameTooLong {
            limit: MAX_RESPONSE_HEADER_NAME_BYTES,
        });
    }
    if headers
        .values()
        .any(|value| value.as_bytes().len() > MAX_HEADER_VALUE_BYTES)
    {
        return Err(ResponseBoundsError::HeaderValueTooLong {
            limit: MAX_HEADER_VALUE_BYTES,
        });
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportError {
    MockHarness(String),
    ResponseBounds(ResponseBoundsError),
}

/// One private implementation will own the production HTTP client (X2).
///
/// Returning a future rather than spelling this as `async fn` keeps callers
/// generic while making the future's `Send` guarantee explicit.
pub trait Transport: Send + Sync + 'static {
    fn send(
        &self,
        request: WireRequest,
    ) -> impl Future<Output = Result<WireResponse, TransportError>> + Send;
}
