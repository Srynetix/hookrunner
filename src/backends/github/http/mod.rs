pub mod middleware;

use axum::{
    extract::Extension,
    http::{header::HeaderName, HeaderMap, HeaderValue},
};
use serde::Deserialize;

use crate::{
    config::Config,
    error::ErrorCode,
    git::{GitBackend, RefType, RepoCloner, RepositoryPath},
    service::ServiceHandler,
};
use serde_json::Value;

use super::{PingEvent, PushEvent};

fn pretty_print_json(s: &str) -> String {
    serde_json::from_str::<Value>(s)
        .and_then(|n| serde_json::to_string_pretty(&n))
        .unwrap_or_default()
}

#[tracing::instrument(skip(config), fields(body_pretty = %pretty_print_json(&body)))]
pub async fn webhook(
    headers: HeaderMap,
    body: String,
    config: Extension<Config>,
    services: Extension<ServiceHandler>,
) -> Result<(HeaderMap, String), ErrorCode> {
    let event = headers
        .get("x-github-event")
        .ok_or(ErrorCode::MissingEventHeader)?;

    match event
        .to_str()
        .map_err(|_| ErrorCode::MalformedEventHeader)?
    {
        "ping" => handle_ping_event(&config, &services, parse_body(&body)?).await,
        "push" => handle_push_event(&config, &services, parse_body(&body)?).await,
        other => Err(ErrorCode::UnsupportedEventHeader(other.to_string())),
    }
}

fn parse_body<'a, T: Deserialize<'a>>(body: &'a str) -> Result<T, ErrorCode> {
    serde_json::from_str(body).map_err(ErrorCode::MalformedEventBody)
}

#[tracing::instrument]
async fn handle_push_event<'a>(
    config: &Config,
    services: &ServiceHandler,
    push_event: PushEvent<'a>,
) -> Result<(HeaderMap, String), ErrorCode> {
    let mut header_map = HeaderMap::new();
    header_map.insert(
        HeaderName::from_static("content-type"),
        HeaderValue::from_static("application/json"),
    );
    let branch = RefType::try_from(push_event.reference)
        .map_err(|e| ErrorCode::MalformedEventBodyField("ref".into(), e.to_string()))?;
    let repository_path = RepositoryPath::new(push_event.repository.full_name).map_err(|e| {
        ErrorCode::MalformedEventBodyField("repository.full_name".into(), e.to_string())
    })?;

    RepoCloner::create_or_update_using_config(
        config,
        services,
        GitBackend::GitHub,
        &repository_path,
        branch,
    )
    .await
    .map_err(|e| ErrorCode::UnhandledError(e.to_string()))?;

    Ok((header_map, serde_json::to_string(&push_event).unwrap()))
}

#[tracing::instrument]
async fn handle_ping_event<'a>(
    config: &Config,
    services: &ServiceHandler,
    ping_event: PingEvent<'a>,
) -> Result<(HeaderMap, String), ErrorCode> {
    let mut header_map = HeaderMap::new();
    header_map.insert(
        HeaderName::from_static("content-type"),
        HeaderValue::from_static("application/json"),
    );
    Ok((header_map, serde_json::to_string(&ping_event).unwrap()))
}

#[cfg(test)]
mod tests {
    use std::{
        any::Any,
        path::{Path, PathBuf},
        sync::Arc,
    };

    use assert_matches::assert_matches;
    use async_trait::async_trait;
    use axum::{body::Body, http::Request, response::Response, Router};
    use pretty_assertions::assert_eq;
    use pseudo::Mock;
    use reqwest::StatusCode;
    use serde_json::json;
    use tower::ServiceExt;

    use crate::{
        backends::github::{Commit, CommitUser, Repository},
        config::Config,
        error::ErrorCode,
        git::{GitError, GitService},
        http::build_http_router,
        service::ServiceHandler,
    };

    use super::{handle_push_event, PushEvent};

    #[derive(Debug)]
    struct FakeGitService {
        pub clone_repository: Mock<(PathBuf, String, String, String), Result<String, GitError>>,
        pub pull: Mock<PathBuf, Result<String, GitError>>,
        pub checkout: Mock<(PathBuf, String), Result<String, GitError>>,
        pub fetch: Mock<PathBuf, Result<String, GitError>>,
    }

    impl FakeGitService {
        pub fn new() -> Self {
            Self {
                clone_repository: Mock::new(Ok("OK".into())),
                pull: Mock::new(Ok("OK".into())),
                checkout: Mock::new(Ok("OK".into())),
                fetch: Mock::new(Ok("OK".into())),
            }
        }
    }

    #[async_trait]
    impl GitService for FakeGitService {
        async fn clone_repository(
            &self,
            working_dir: &Path,
            reference: &str,
            repo_path: &str,
            folder_path: &str,
        ) -> Result<String, GitError> {
            self.clone_repository.call((
                working_dir.to_owned(),
                reference.to_owned(),
                repo_path.to_owned(),
                folder_path.to_owned(),
            ))
        }

        async fn pull(&self, working_dir: &Path) -> Result<String, GitError> {
            self.pull.call(working_dir.to_owned())
        }

        async fn checkout(&self, working_dir: &Path, reference: &str) -> Result<String, GitError> {
            self.checkout
                .call((working_dir.to_owned(), reference.to_owned()))
        }

        async fn fetch(&self, working_dir: &Path) -> Result<String, GitError> {
            self.fetch.call(working_dir.to_owned())
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    fn create_test_config() -> Config {
        Config::empty()
    }

    fn create_test_services() -> ServiceHandler {
        ServiceHandler::new(Arc::new(FakeGitService::new()))
    }

    fn create_test_router() -> Router {
        let config = create_test_config();
        let services = create_test_services();
        build_http_router(config, services)
    }

    fn extract_fake_git_service(services: &ServiceHandler) -> &FakeGitService {
        services
            .git()
            .as_any()
            .downcast_ref::<FakeGitService>()
            .unwrap()
    }

    async fn response_to_string<T: hyper::body::HttpBody>(response: Response<T>) -> String
    where
        T::Error: std::fmt::Debug,
    {
        let bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    async fn response_to_json<T: hyper::body::HttpBody>(response: Response<T>) -> serde_json::Value
    where
        T::Error: std::fmt::Debug,
    {
        let string = response_to_string(response).await;
        serde_json::from_str(&string).unwrap()
    }

    fn error_to_json(err: ErrorCode) -> serde_json::Value {
        serde_json::to_value(err.details()).unwrap()
    }

    async fn assert_response_is_error<T: hyper::body::HttpBody>(
        response: Response<T>,
        error: ErrorCode,
    ) where
        T::Error: std::fmt::Debug,
    {
        let data = response_to_json(response).await;
        assert_eq!(data, error_to_json(error));
    }

    #[tokio::test]
    async fn test_handle_push_event_wrong_ref() {
        let config = create_test_config();
        let services = create_test_services();

        let event = PushEvent {
            base_ref: "wrong",
            reference: "wrong",
            head_commit: Commit {
                message: "ooo",
                timestamp: "nope",
            },
            pusher: CommitUser {
                email: "hello@local.test",
                name: "hello",
            },
            repository: Repository {
                full_name: "hello/world",
                name: "world",
            },
        };

        let err = handle_push_event(&config, &services, event)
            .await
            .unwrap_err();
        assert_matches!(err, ErrorCode::MalformedEventBodyField(_, _));
    }

    #[tokio::test]
    async fn test_verify_github_signature_middleware_invalid_user_agent() {
        let app = create_test_router();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/webhook/github")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        assert_response_is_error(response, ErrorCode::InvalidUserAgent).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_verify_github_signature_middleware_invalid_signature() {
        let mut config = create_test_config();
        // This configuration will enable signature verification
        config.set_webhook_secret("secret");
        let services = create_test_services();
        let app = build_http_router(config, services);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .header("User-Agent", "GitHub-Hookshot/value")
                    .uri("/webhook/github")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        assert_response_is_error(response, ErrorCode::InvalidSignature).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_missing_event_header() {
        let app = create_test_router();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .header("User-Agent", "GitHub-Hookshot/value")
                    .uri("/webhook/github")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        assert_response_is_error(response, ErrorCode::MissingEventHeader).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_unsupported_event_header() {
        let app = create_test_router();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .header("User-Agent", "GitHub-Hookshot/value")
                    .header("X-GitHub-Event", "unknown")
                    .uri("/webhook/github")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        assert_response_is_error(
            response,
            ErrorCode::UnsupportedEventHeader("unknown".into()),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_malformed_event_body() {
        let app = create_test_router();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .header("User-Agent", "GitHub-Hookshot/value")
                    .header("X-GitHub-Event", "ping")
                    .uri("/webhook/github")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        let data = response_to_json(response).await;
        assert_eq!(
            data,
            json!({
                "internal_code": 6,
                "message": "Malformed event body: 'EOF while parsing a value at line 1 column 0'"
            })
        );
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_malformed_event_body_field_ref() {
        let app = create_test_router();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .header("User-Agent", "GitHub-Hookshot/value")
                    .header("X-GitHub-Event", "push")
                    .uri("/webhook/github")
                    .body(
                        json!({
                            "ref": "sample",
                            "base_ref": "sample",
                            "head_commit": {
                                "message": "sample",
                                "timestamp": "sample"
                            },
                            "repository": {
                                "full_name": "sample",
                                "name": "sample"
                            },
                            "pusher": {
                                "name": "sample",
                                "email": "sample"
                            }
                        })
                        .to_string()
                        .into(),
                    )
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        assert_response_is_error(
            response,
            ErrorCode::MalformedEventBodyField(
                "ref".into(),
                "Unsupported Git reference type: sample".into(),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_malformed_event_body_field_repo() {
        let app = create_test_router();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .header("User-Agent", "GitHub-Hookshot/value")
                    .header("X-GitHub-Event", "push")
                    .uri("/webhook/github")
                    .body(
                        json!({
                            "ref": "refs/branches/sample",
                            "base_ref": "refs/branches/sample",
                            "head_commit": {
                                "message": "sample",
                                "timestamp": "sample"
                            },
                            "repository": {
                                "full_name": "sample",
                                "name": "sample"
                            },
                            "pusher": {
                                "name": "sample",
                                "email": "sample"
                            }
                        })
                        .to_string()
                        .into(),
                    )
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        assert_response_is_error(
            response,
            ErrorCode::MalformedEventBodyField(
                "repository.full_name".into(),
                "Malformed repository path: sample".into(),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_push_event_clone() {
        let config = create_test_config();
        let services = create_test_services();
        let app = build_http_router(config, services.clone());

        let json_data = json!({
            "ref": "refs/branches/sample",
            "base_ref": "refs/branches/sample",
            "head_commit": {
                "message": "sample",
                "timestamp": "sample"
            },
            "repository": {
                "full_name": "Srynetix/things",
                "name": "things"
            },
            "pusher": {
                "name": "sample",
                "email": "sample"
            }
        });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .header("User-Agent", "GitHub-Hookshot/value")
                    .header("X-GitHub-Event", "push")
                    .uri("/webhook/github")
                    .body(json_data.to_string().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        let data = response_to_json(response).await;
        assert_eq!(data, json_data);

        let fake_git_service = extract_fake_git_service(&services);
        assert!(fake_git_service.clone_repository.called());
        assert!(!fake_git_service.fetch.called());
        assert!(!fake_git_service.checkout.called());
        assert!(!fake_git_service.pull.called());

        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_push_event_checkout() {
        let tempdir = tempfile::tempdir().unwrap();
        std::fs::create_dir(tempdir.path().join("things")).unwrap();

        let mut config = create_test_config();
        config.set_working_dir(tempdir.path());

        let services = create_test_services();
        let app = build_http_router(config, services.clone());

        let json_data = json!({
            "ref": "refs/branches/sample",
            "base_ref": "refs/branches/sample",
            "head_commit": {
                "message": "sample",
                "timestamp": "sample"
            },
            "repository": {
                "full_name": "Srynetix/things",
                "name": "things"
            },
            "pusher": {
                "name": "sample",
                "email": "sample"
            }
        });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .header("User-Agent", "GitHub-Hookshot/value")
                    .header("X-GitHub-Event", "push")
                    .uri("/webhook/github")
                    .body(json_data.to_string().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        let data = response_to_json(response).await;
        assert_eq!(data, json_data);

        let fake_git_service = extract_fake_git_service(&services);
        assert!(!fake_git_service.clone_repository.called());
        assert!(fake_git_service.fetch.called());
        assert!(fake_git_service.checkout.called());
        assert!(fake_git_service.pull.called());

        assert_eq!(status, StatusCode::OK);
    }
}
