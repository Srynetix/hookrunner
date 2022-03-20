use clap::Parser;

use crate::git::{GitBackend, RefType, RepositoryPath};

/// Execute actions on Git hosting webhooks
#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Args {
    /// Telemetry URL (disabled as default)
    #[clap(long)]
    pub telemetry_url: Option<String>,

    /// GitHub API URL (https://api.github.com as default)
    #[clap(long)]
    pub github_api_url: Option<String>,

    /// Working directory (current directory as default)
    #[clap(long)]
    pub working_dir: Option<String>,

    /// Webhook secret (disabled as default)
    #[clap(long)]
    pub webhook_secret: Option<String>,

    /// Repository mapping configuration
    #[clap(long)]
    pub repo_mapping: Option<String>,

    /// Bind IP
    #[clap(long)]
    pub bind_ip: Option<String>,

    /// Command
    #[clap(subcommand)]
    pub command: SubCommand,
}

#[derive(Parser, Debug)]
pub enum SubCommand {
    /// Run server
    Run,
    /// Install webhook
    Install(InstallCommand),
    /// Uninstall webhook
    Uninstall(InstallCommand),
    /// Synchronize
    Synchronize(SynchronizeCommand),
}

#[derive(Parser, Debug)]
pub struct InstallCommand {
    /// Git hosting backend
    #[clap(long, default_value = "github")]
    pub backend: GitBackend,

    /// Repository full name
    #[clap(long)]
    pub repository: RepositoryPath,

    /// URL to register
    #[clap(long)]
    pub url: String,

    /// API username
    #[clap(long)]
    pub username: String,

    /// API token
    #[clap(long)]
    pub token: String,
}

#[derive(Parser, Debug)]
pub struct UninstallCommand {
    /// Git hosting backend
    #[clap(long, default_value = "github")]
    pub backend: GitBackend,

    /// Repository full name
    #[clap(long)]
    pub repository: RepositoryPath,

    /// URL to unregister
    #[clap(long)]
    pub url: String,

    /// API username
    #[clap(long)]
    pub username: String,

    /// API token
    #[clap(long)]
    pub token: String,
}

#[derive(Parser, Debug)]
pub struct SynchronizeCommand {
    /// Git hosting backend
    #[clap(long, default_value = "github")]
    pub backend: GitBackend,

    /// Repository full name
    #[clap(long)]
    pub repository: RepositoryPath,

    /// Git reference name (e.g. refs/branches/my-branch or refs/tags/my-tag)
    #[clap(name = "ref", long)]
    pub reference: RefType,
}
