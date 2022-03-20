use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitHubError {
    #[error("error while registering webhook.")]
    CouldNotRegisterWebhook(#[source] reqwest::Error),

    #[error("error while listing webhooks.")]
    CouldNotListWebhooks(#[source] reqwest::Error),

    #[error("error while unregistering webhook.")]
    CouldNotUnregisterWebhook(#[source] reqwest::Error),

    #[error("error code received from GitHub.")]
    BadStatusCode(#[source] reqwest::Error),

    #[error("error while parsing GitHub response.")]
    MalformedResponse(#[source] reqwest::Error),
}
