use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{
    config::Config,
    server_info::{APP_NAME, APP_VERSION},
};

use super::error::GitHubError;

pub struct Client {
    username: String,
    token: String,
}

#[derive(Deserialize)]
pub struct WebhookConfig {
    url: String,
}

#[derive(Deserialize)]
pub struct Webhook {
    id: u32,
    config: WebhookConfig,
}

impl Client {
    pub fn new<T: Into<String>>(username: T, token: T) -> Self {
        Self {
            username: username.into(),
            token: token.into(),
        }
    }

    pub async fn try_register_webhook(
        &self,
        config: &Config,
        owner: &str,
        repo: &str,
        url: &str,
    ) -> Result<u32, GitHubError> {
        if let Some(u) = self.check_webhook_url(config, owner, repo, url).await? {
            tracing::warn!(
                id = u,
                message = "Webhook already registered",
                owner = owner,
                repo = repo,
                url = url
            );
            Ok(u)
        } else {
            Ok(self.register_webhook(config, owner, repo, url).await?.id)
        }
    }

    pub async fn try_unregister_webhook(
        &self,
        config: &Config,
        owner: &str,
        repo: &str,
        url: &str,
    ) -> Result<(), GitHubError> {
        if let Some(u) = self.check_webhook_url(config, owner, repo, url).await? {
            self.unregister_webhook(config, owner, repo, u).await
        } else {
            tracing::error!(
                message = "Unknown webhook",
                owner = owner,
                repo = repo,
                url = url
            );
            Ok(())
        }
    }

    fn create_client(&self) -> reqwest::Client {
        reqwest::ClientBuilder::new()
            .connect_timeout(Duration::from_secs(10))
            .user_agent(format!("{APP_NAME}/{APP_VERSION}"))
            .build()
            .unwrap()
    }

    async fn register_webhook(
        &self,
        config: &Config,
        owner: &str,
        repo: &str,
        url: &str,
    ) -> Result<Webhook, GitHubError> {
        #[derive(Serialize)]
        struct WebhookConfig {
            url: String,
            content_type: &'static str,
            secret: Option<String>,
        }

        #[derive(Serialize)]
        struct Data {
            name: &'static str,
            config: WebhookConfig,
            events: &'static [&'static str],
        }

        let data = Data {
            name: "web",
            config: WebhookConfig {
                url: url.to_owned(),
                content_type: "json",
                secret: None,
            },
            events: &["push"],
        };

        let root_url = config.github_api_url();
        let resp = self
            .create_client()
            .post(format!("{root_url}/repos/{owner}/{repo}/hooks"))
            .basic_auth(&self.username, Some(&self.token))
            .json(&data)
            .send()
            .await
            .map_err(GitHubError::CouldNotRegisterWebhook)?;

        let webhook: Webhook = resp
            .error_for_status()
            .map_err(GitHubError::BadStatusCode)?
            .json()
            .await
            .map_err(GitHubError::MalformedResponse)?;

        tracing::info!(
            id = webhook.id,
            message = "New webhook installed",
            owner = owner,
            repo = repo,
            url = url
        );

        Ok(webhook)
    }

    async fn list_webhooks(
        &self,
        config: &Config,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<Webhook>, GitHubError> {
        let root_url = config.github_api_url();
        let resp = self
            .create_client()
            .get(format!("{root_url}/repos/{owner}/{repo}/hooks"))
            .basic_auth(&self.username, Some(&self.token))
            .send()
            .await
            .map_err(GitHubError::CouldNotListWebhooks)?;

        let data: Vec<Webhook> = resp
            .error_for_status()
            .map_err(GitHubError::BadStatusCode)?
            .json()
            .await
            .map_err(GitHubError::MalformedResponse)?;
        Ok(data)
    }

    async fn check_webhook_url(
        &self,
        config: &Config,
        owner: &str,
        repo: &str,
        url: &str,
    ) -> Result<Option<u32>, GitHubError> {
        Ok(self
            .list_webhooks(config, owner, repo)
            .await?
            .into_iter()
            .find(|w| w.config.url == url)
            .map(|w| w.id))
    }

    async fn unregister_webhook(
        &self,
        config: &Config,
        owner: &str,
        repo: &str,
        id: u32,
    ) -> Result<(), GitHubError> {
        let root_url = config.github_api_url();
        let resp = self
            .create_client()
            .delete(format!("{root_url}/repos/{owner}/{repo}/hooks/{id}"))
            .basic_auth(&self.username, Some(&self.token))
            .send()
            .await
            .map_err(GitHubError::CouldNotUnregisterWebhook)?;

        resp.error_for_status()
            .map_err(GitHubError::BadStatusCode)?;

        tracing::info!(
            id = id,
            message = "Webhook unregistered",
            owner = owner,
            repo = repo,
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::config::Config;
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use wiremock::{matchers, Mock, MockServer, ResponseTemplate};

    use super::Client;

    async fn test_config() -> (MockServer, Config) {
        let mut config = Config::empty();
        let server = MockServer::start().await;
        config.set_github_api_url(server.uri());

        (server, config)
    }

    #[tokio::test]
    async fn test_register_webhook() {
        let (server, config) = test_config().await;
        let client = Client::new("username", "token");

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/repos/owner/repo/hooks"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": 1234u32,
                "config": {
                    "url": "url"
                }
            })))
            .expect(1)
            .mount(&server)
            .await;

        assert_eq!(
            client
                .register_webhook(&config, "owner", "repo", "url")
                .await
                .unwrap()
                .id,
            1234
        );
    }

    #[tokio::test]
    async fn test_list_webhook() {
        let (server, config) = test_config().await;
        let client = Client::new("username", "token");

        Mock::given(matchers::method("GET"))
            .and(matchers::path("/repos/owner/repo/hooks"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    "id": 1234u32,
                    "config": {
                        "url": "url"
                    }
                }
            ])))
            .expect(1)
            .mount(&server)
            .await;

        assert_eq!(
            client
                .list_webhooks(&config, "owner", "repo")
                .await
                .unwrap()
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn test_unregister_webhook() {
        let (server, config) = test_config().await;
        let client = Client::new("username", "token");

        Mock::given(matchers::method("DELETE"))
            .and(matchers::path("/repos/owner/repo/hooks/1234"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        client
            .unregister_webhook(&config, "owner", "repo", 1234)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_try_register_webhook_absent() {
        let (server, config) = test_config().await;
        let client = Client::new("username", "token");

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/repos/owner/repo/hooks"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": 1234,
                "config": {
                    "url": "url"
                }
            })))
            .expect(1)
            .mount(&server)
            .await;

        Mock::given(matchers::method("GET"))
            .and(matchers::path("/repos/owner/repo/hooks"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    "id": 5678,
                    "config": {
                        "url": "other-url"
                    }
                }
            ])))
            .expect(1)
            .mount(&server)
            .await;

        assert_eq!(
            client
                .try_register_webhook(&config, "owner", "repo", "url")
                .await
                .unwrap(),
            1234
        );
    }

    #[tokio::test]
    async fn test_try_register_webhook_present() {
        let (server, config) = test_config().await;
        let client = Client::new("username", "token");

        Mock::given(matchers::method("GET"))
            .and(matchers::path("/repos/owner/repo/hooks"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    "id": 1234u32,
                    "config": {
                        "url": "url"
                    }
                }
            ])))
            .expect(1)
            .mount(&server)
            .await;

        assert_eq!(
            client
                .try_register_webhook(&config, "owner", "repo", "url")
                .await
                .unwrap(),
            1234
        );
    }

    #[tokio::test]
    async fn test_try_unregister_webhook_present() {
        let (server, config) = test_config().await;
        let client = Client::new("username", "token");

        Mock::given(matchers::method("DELETE"))
            .and(matchers::path("/repos/owner/repo/hooks/1234"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        Mock::given(matchers::method("GET"))
            .and(matchers::path("/repos/owner/repo/hooks"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    "id": 1234,
                    "config": {
                        "url": "url"
                    }
                }
            ])))
            .expect(1)
            .mount(&server)
            .await;

        client
            .try_unregister_webhook(&config, "owner", "repo", "url")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_try_unregister_webhook_absent() {
        let (server, config) = test_config().await;
        let client = Client::new("username", "token");

        Mock::given(matchers::method("GET"))
            .and(matchers::path("/repos/owner/repo/hooks"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    "id": 5678,
                    "config": {
                        "url": "other-url"
                    }
                }
            ])))
            .expect(1)
            .mount(&server)
            .await;

        client
            .try_unregister_webhook(&config, "owner", "repo", "url")
            .await
            .unwrap();
    }
}
