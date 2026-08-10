//! The single wire hand-off seam shared by the future actor and the mock.

use std::future::Future;

use http::{Request, Response};

pub type WireRequest = Request<Vec<u8>>;
pub type WireResponse = Response<Vec<u8>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportError {
    MockHarness(String),
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
