use std::{
    any::Any,
    path::{Path, PathBuf},
};

use super::GitError;
use async_trait::async_trait;
use tokio::process::Command;
use which::which;

#[async_trait]
pub trait GitService: std::fmt::Debug + Send + Sync {
    async fn clone_repository(
        &self,
        working_dir: &Path,
        reference: &str,
        url: &str,
        folder_name: &str,
    ) -> Result<String, GitError>;
    async fn fetch(&self, working_dir: &Path) -> Result<String, GitError>;
    async fn checkout(&self, working_dir: &Path, reference: &str) -> Result<String, GitError>;
    async fn pull(&self, working_dir: &Path) -> Result<String, GitError>;

    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug)]
pub struct GitExecutable {
    binary_path: PathBuf,
}

impl GitExecutable {
    pub fn new() -> Result<Self, GitError> {
        let binary_path = which("git").map_err(|_| GitError::MissingGitBinary)?;

        Ok(Self { binary_path })
    }

    #[tracing::instrument(skip(self))]
    async fn execute(
        &self,
        working_directory: &Path,
        command: &str,
        args: &[&str],
    ) -> Result<String, GitError> {
        let output = Command::new(&self.binary_path)
            .arg(command)
            .args(args)
            .current_dir(working_directory)
            .output()
            .await
            .map_err(|e| GitError::GitExecutionError(e.to_string()))?;

        if output.status.success() {
            let string_output = String::from_utf8_lossy(&output.stdout).trim().to_string();
            tracing::info!(
                return_code = output.status.code(),
                stdout = %string_output
            );
            Ok(string_output)
        } else {
            let string_output = String::from_utf8_lossy(&output.stderr).trim().to_string();
            tracing::error!(
                return_code = output.status.code(),
                stderr = %string_output
            );
            Err(GitError::GitExecutionError(string_output))
        }
    }
}

#[async_trait]
impl GitService for GitExecutable {
    async fn clone_repository(
        &self,
        working_dir: &Path,
        reference: &str,
        url: &str,
        folder_name: &str,
    ) -> Result<String, GitError> {
        self.execute(working_dir, "clone", &["-b", reference, url, folder_name])
            .await
    }

    async fn fetch(&self, working_dir: &Path) -> Result<String, GitError> {
        self.execute(working_dir, "fetch", &[]).await
    }

    async fn checkout(&self, working_dir: &Path, reference: &str) -> Result<String, GitError> {
        self.execute(working_dir, "checkout", &[reference]).await
    }

    async fn pull(&self, working_dir: &Path) -> Result<String, GitError> {
        self.execute(working_dir, "pull", &[]).await
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
