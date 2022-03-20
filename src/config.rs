use std::{collections::HashMap, net::SocketAddr, path::PathBuf, str::FromStr};

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Missing working directory: '{0}'. Make sure it exists on disk.")]
    MissingWorkingDirectory(PathBuf),
    #[error("Malformed bind IP: '{0}'. Make sure you entered a valid IP.")]
    MalformedBindIp(String),
}

#[derive(Debug, Clone)]
pub struct Config {
    telemetry_url: Option<String>,
    github_api_url: String,
    webhook_secret: Option<String>,
    working_dir: Option<String>,
    repo_mapping: HashMap<String, PathBuf>,
    bind_ip: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            telemetry_url: env_to_str("HR_TELEMETRY_URL"),
            github_api_url: env_to_str("HR_GITHUB_API_URL")
                .unwrap_or_else(|| "https://api.github.com".into()),
            webhook_secret: env_to_str("HR_WEBHOOK_SECRET"),
            working_dir: env_to_str("HR_WORKING_DIR"),
            repo_mapping: env_to_repo_mapping("HR_REPO_MAPPING"),
            bind_ip: env_to_str("HR_BIND_IP").unwrap_or_else(|| "127.0.0.1:3000".into()),
        }
    }

    pub fn empty() -> Self {
        Self {
            telemetry_url: None,
            github_api_url: "".into(),
            webhook_secret: None,
            working_dir: None,
            repo_mapping: HashMap::new(),
            bind_ip: "".into(),
        }
    }

    pub fn github_api_url(&self) -> &str {
        &self.github_api_url
    }

    pub fn repo_mapping(&self) -> &HashMap<String, PathBuf> {
        &self.repo_mapping
    }

    pub fn working_dir(&self) -> Option<&str> {
        self.working_dir.as_deref()
    }

    pub fn telemetry_url(&self) -> Option<&str> {
        self.telemetry_url.as_deref()
    }

    pub fn webhook_secret(&self) -> Option<&str> {
        self.webhook_secret.as_deref()
    }

    pub fn bind_ip(&self) -> &str {
        &self.bind_ip
    }

    pub fn set_github_api_url<T: Into<String>>(&mut self, value: T) {
        self.github_api_url = value.into();
    }

    pub fn set_working_dir<T: Into<String>>(&mut self, value: T) {
        self.working_dir = Some(value.into());
    }

    pub fn set_telemetry_url<T: Into<String>>(&mut self, value: T) {
        self.telemetry_url = Some(value.into());
    }

    pub fn set_webhook_secret<T: Into<String>>(&mut self, value: T) {
        self.webhook_secret = Some(value.into());
    }

    pub fn set_repo_mapping(&mut self, conf: &str) {
        self.repo_mapping = parse_repo_mapping(conf);
    }

    pub fn set_bind_ip<T: Into<String>>(&mut self, value: T) {
        self.bind_ip = value.into();
    }

    pub fn validate_configuration(&self) -> Result<(), ConfigError> {
        // Check if working directory exists
        if let Some(w) = &self.working_dir {
            let path = PathBuf::from(w);
            if !path.exists() {
                return Err(ConfigError::MissingWorkingDirectory(path));
            }
        }

        let _ = SocketAddr::from_str(&self.bind_ip)
            .map_err(|_| ConfigError::MalformedBindIp(self.bind_ip.clone()))?;

        Ok(())
    }
}

fn env_to_str(env_key: &str) -> Option<String> {
    std::env::var(env_key).ok().filter(|s| !s.is_empty())
}

/// Convert environment value to repository mapping.
/// Syntax is like that:
///
/// ```text
/// org/repo-name=./local/folder,org2/repo-name2=./target/folder
/// ```
fn env_to_repo_mapping(env_key: &str) -> HashMap<String, PathBuf> {
    match env_to_str(env_key) {
        Some(value) => parse_repo_mapping(&value),
        None => HashMap::new(),
    }
}

fn parse_repo_mapping(conf: &str) -> HashMap<String, PathBuf> {
    HashMap::from_iter(conf.split(',').map(|entry| {
        let entry_split = entry.split('=').collect::<Vec<_>>();
        (entry_split[0].to_owned(), PathBuf::from(entry_split[1]))
    }))
}
