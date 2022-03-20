use std::str::FromStr;

use super::GitError;

#[derive(Debug)]
pub enum RefType {
    Branch(String),
    Tag(String),
}

impl ToString for RefType {
    fn to_string(&self) -> String {
        match &self {
            Self::Branch(b) => b.into(),
            Self::Tag(t) => t.into(),
        }
    }
}

impl TryFrom<&str> for RefType {
    type Error = GitError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.starts_with("refs/tags/") {
            return Ok(Self::Tag(value.strip_prefix("refs/tags/").unwrap().into()));
        } else if value.starts_with("refs/branches/") {
            return Ok(Self::Branch(
                value.strip_prefix("refs/branches/").unwrap().into(),
            ));
        } else {
            Err(GitError::UnsupportedRefType(value.into()))
        }
    }
}

impl FromStr for RefType {
    type Err = GitError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.starts_with("refs/tags/") {
            return Ok(Self::Tag(value.strip_prefix("refs/tags/").unwrap().into()));
        } else if value.starts_with("refs/branches/") {
            return Ok(Self::Branch(
                value.strip_prefix("refs/branches/").unwrap().into(),
            ));
        } else {
            Err(GitError::UnsupportedRefType(value.into()))
        }
    }
}
