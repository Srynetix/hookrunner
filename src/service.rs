use std::sync::Arc;

use crate::git::{GitError, GitExecutable, GitService};

#[derive(Debug, Clone)]
pub struct ServiceHandler {
    git_service: Arc<dyn GitService>,
}

impl ServiceHandler {
    pub fn new(git_service: Arc<dyn GitService>) -> Self {
        Self { git_service }
    }

    pub fn new_defaults() -> Result<Self, GitError> {
        Ok(Self {
            git_service: Arc::new(GitExecutable::new()?),
        })
    }

    pub fn git(&self) -> &dyn GitService {
        self.git_service.as_ref()
    }
}
