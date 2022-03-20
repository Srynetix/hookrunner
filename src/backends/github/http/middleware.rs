use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use axum::{
    body::{Body, HttpBody},
    http::{Method, Request},
    response::{IntoResponse, Response},
};

use tower::{Layer, Service};

use crate::{crypto::is_valid_signature, error::ErrorCode};

const GITHUB_SIGNATURE_HEADER: &str = "X-Hub-Signature-256";
const SIGNATURE_PREFIX: &str = "sha256=";
const GITHUB_USER_AGENT: &str = "GitHub-Hookshot/";

pub struct VerifyGitHubSignatureLayer {
    secret: Option<String>,
}

impl VerifyGitHubSignatureLayer {
    pub fn new(secret: Option<String>) -> Self {
        Self { secret }
    }
}

impl<S> Layer<S> for VerifyGitHubSignatureLayer {
    type Service = VerifyGitHubSignatureMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        VerifyGitHubSignatureMiddleware::new(self.secret.clone(), inner)
    }
}

#[derive(Clone)]
pub struct VerifyGitHubSignatureMiddleware<S> {
    secret: Option<String>,
    inner: S,
}

impl<S> VerifyGitHubSignatureMiddleware<S> {
    pub fn new(secret: Option<String>, inner: S) -> Self {
        Self { secret, inner }
    }
}

type BoxFuture<'a, Output> = Pin<Box<dyn Future<Output = Output> + Send + 'a>>;

impl<S> Service<Request<Body>> for VerifyGitHubSignatureMiddleware<S>
where
    S: Service<Request<Body>, Response = Response> + Send + 'static + Clone,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request<Body>) -> Self::Future {
        let mut inner = self.inner.clone();
        let secret = self.secret.clone();
        let fut = async move {
            if request.method() == Method::POST {
                let headers = request.headers();

                // Check for useragent
                if headers
                    .get("User-Agent")
                    .and_then(|v| v.to_str().ok())
                    .filter(|v| v.starts_with(GITHUB_USER_AGENT))
                    .is_none()
                {
                    return Ok(ErrorCode::InvalidUserAgent.into_response());
                }

                // Check for signature
                if let Some(secret) = secret {
                    if let Some(v) = headers.get(GITHUB_SIGNATURE_HEADER) {
                        let signature = v
                            .to_str()
                            .ok()
                            .and_then(|x| x.strip_prefix(SIGNATURE_PREFIX))
                            .unwrap_or_default()
                            .to_string();

                        let mut body = Vec::<u8>::new();
                        let request_body = request.body_mut();
                        while let Some(d) = request_body.data().await {
                            body.extend(d.unwrap());
                        }

                        if !is_valid_signature(&signature, &body, &secret) {
                            return Ok(ErrorCode::InvalidSignature.into_response());
                        }

                        *request.body_mut() = body.into();
                    } else {
                        return Ok(ErrorCode::InvalidSignature.into_response());
                    }
                }
            }

            let future = inner.call(request);
            let response: Response = future.await?;
            Ok(response)
        };

        Box::pin(fut)
    }
}
