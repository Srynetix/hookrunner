use std::{borrow::Cow, str::FromStr};

use super::GitError;

#[derive(Debug)]
pub enum GitBackend {
    GitHub,
    GitLab,
    Custom(String),
}

impl GitBackend {
    pub fn root_url(&self) -> Cow<str> {
        match self {
            Self::GitHub => Cow::Borrowed("https://github.com"),
            Self::GitLab => Cow::Borrowed("https://gitlab.com"),
            Self::Custom(url) => Cow::Owned(url.into()),
        }
    }
}

impl FromStr for GitBackend {
    type Err = GitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_lowercase()[..] {
            "github" => Ok(Self::GitHub),
            "gitlab" => Ok(Self::GitLab),
            other => match s.strip_prefix("custom:") {
                Some(url) => Ok(Self::Custom(url.into())),
                None => Err(GitError::UnsupportedGitBackend(other.into())),
            },
        }
    }
}
