use actix_service::{Service, Transform};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::{Error, HttpMessage};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::task::{Context, Poll};
use uuid::Uuid;

/// HTTP header name used for request identification.
static REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

/// Request identifier stored in request extensions.
#[derive(Clone)]
pub struct RequestId(pub String);

/// Request ID middleware.
///
/// Attaches a request ID to each incoming request and
/// propagates it via the `x-request-id` header.
pub struct RequestIdMiddleware;

/// Request ID middleware service.
pub struct RequestIdService<S> {
    service: S,
}

impl<S, B> Transform<S, ServiceRequest> for RequestIdMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RequestIdService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    /// Creates a new request ID service.
    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestIdService { service }))
    }
}

impl<S, B> Service<ServiceRequest> for RequestIdService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    /// Checks whether the underlying service is ready.
    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    /// Processes an incoming request.
    ///
    /// Generates a request ID if missing and adds it to both
    /// request extensions and response headers.
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let request_id = req
            .headers()
            .get(&REQUEST_ID_HEADER)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_owned())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        req.extensions_mut().insert(RequestId(request_id.clone()));

        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;
            res.response_mut().headers_mut().insert(
                REQUEST_ID_HEADER.clone(),
                HeaderValue::from_str(&request_id).unwrap(),
            );
            Ok(res)
        })
    }
}
