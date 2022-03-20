use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum GitError {
    #[error("Missing Git binary. Make sure Git is installed on your system and is globally accessible (present in PATH).")]
    MissingGitBinary,
    #[error("Error while executing git: {0}")]
    GitExecutionError(String),
    #[error("I/O error: {0}")]
    IoError(String),
    #[error("Unsupported Git reference type: {0}")]
    UnsupportedRefType(String),
    #[error("Unsupported Git backend: {0}")]
    UnsupportedGitBackend(String),
    #[error("Malformed repository path: {0}")]
    MalformedRepositoryPath(String),
}
