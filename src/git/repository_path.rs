use std::str::FromStr;

use super::GitError;

#[derive(Debug)]
pub struct RepositoryPath {
    owner: String,
    name: String,
}

impl RepositoryPath {
    pub fn new(path: &str) -> Result<Self, GitError> {
        let (owner, name) = Self::split_repo_path(path)?;

        Ok(Self {
            owner: owner.into(),
            name: name.into(),
        })
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn full_name(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }

    fn split_repo_path(repo_path: &str) -> Result<(&str, &str), GitError> {
        let split: Vec<_> = repo_path.split('/').collect();

        if split.len() == 2 {
            Ok((split[0], split[1]))
        } else {
            Err(GitError::MalformedRepositoryPath(repo_path.into()))
        }
    }
}

impl FromStr for RepositoryPath {
    type Err = GitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}
