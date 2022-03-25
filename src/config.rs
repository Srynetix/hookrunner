use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    net::SocketAddr,
    path::{Path, PathBuf},
    str::FromStr,
};
use url::Url;

static DEFAULT_URL: Lazy<Url> = Lazy::new(|| Url::parse("http://localhost").unwrap());
static DEFAULT_GITHUB_API_URL: Lazy<Url> =
    Lazy::new(|| Url::parse("https://api.github.com").unwrap());
static DEFAULT_BIND_IP: Lazy<SocketAddr> =
    Lazy::new(|| SocketAddr::from_str("0.0.0.0:3000").unwrap());

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Missing working directory: '{0}'. Make sure it exists on disk.")]
    MissingWorkingDirectory(PathBuf),
}

#[derive(Debug, Clone)]
pub struct Config {
    telemetry_url: Option<Url>,
    github_api_url: Url,
    webhook_secret: Option<String>,
    working_dir: Option<PathBuf>,
    repo_mapping: HashMap<String, PathBuf>,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            telemetry_url: env_to_url("HR_TELEMETRY_URL"),
            github_api_url: env_to_url("HR_GITHUB_API_URL")
                .unwrap_or_else(|| DEFAULT_GITHUB_API_URL.clone()),
            webhook_secret: env_to_str("HR_WEBHOOK_SECRET"),
            working_dir: env_to_pathbuf("HR_WORKING_DIR"),
            repo_mapping: env_to_repo_mapping("HR_REPO_MAPPING"),
        }
    }

    pub fn empty() -> Self {
        Self {
            telemetry_url: None,
            github_api_url: DEFAULT_URL.clone(),
            webhook_secret: None,
            working_dir: None,
            repo_mapping: HashMap::new(),
        }
    }

    pub fn github_api_url(&self) -> &Url {
        &self.github_api_url
    }

    pub fn repo_mapping(&self) -> &HashMap<String, PathBuf> {
        &self.repo_mapping
    }

    pub fn working_dir(&self) -> Option<&Path> {
        self.working_dir.as_deref()
    }

    pub fn telemetry_url(&self) -> Option<&Url> {
        self.telemetry_url.as_ref()
    }

    pub fn webhook_secret(&self) -> Option<&str> {
        self.webhook_secret.as_deref()
    }

    pub fn set_github_api_url(&mut self, value: Url) {
        self.github_api_url = value;
    }

    pub fn set_working_dir<T: AsRef<Path>>(&mut self, value: T) {
        self.working_dir = Some(value.as_ref().to_owned());
    }

    pub fn set_telemetry_url(&mut self, value: Url) {
        self.telemetry_url = Some(value);
    }

    pub fn set_webhook_secret<T: Into<String>>(&mut self, value: T) {
        self.webhook_secret = Some(value.into());
    }

    pub fn set_repo_mapping(&mut self, conf: &str) {
        self.repo_mapping = parse_repo_mapping(conf);
    }

    pub fn validate_configuration(&self) -> Result<(), ConfigError> {
        // Check if working directory exists
        if let Some(w) = &self.working_dir {
            let path = PathBuf::from(w);
            if !path.exists() {
                return Err(ConfigError::MissingWorkingDirectory(path));
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct ServerConfig {
    bind_ip: SocketAddr,
}

impl ServerConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let bind_ip = env_to_str("HR_BIND_IP")
            .and_then(|v| {
                SocketAddr::from_str(&v[..])
                    .map_err(|e| {
                        tracing::error!(
                            "error while parsing bind ip '{}' from environment variable HR_BIND_IP, will use default value '{}'",
                            v,
                            *DEFAULT_BIND_IP
                        );
                        e
                    })
                    .ok()
            })
            .unwrap_or(*DEFAULT_BIND_IP);

        Ok(Self { bind_ip })
    }

    pub fn empty() -> Self {
        Self {
            bind_ip: *DEFAULT_BIND_IP,
        }
    }

    pub fn bind_ip(&self) -> &SocketAddr {
        &self.bind_ip
    }

    pub fn set_bind_ip(&mut self, value: SocketAddr) {
        self.bind_ip = value;
    }
}

fn env_to_str(env_key: &str) -> Option<String> {
    std::env::var(env_key).ok().filter(|s| !s.is_empty())
}

fn env_to_url(env_key: &str) -> Option<Url> {
    env_to_str(env_key).and_then(|x| Url::from_str(&x[..]).ok())
}

fn env_to_pathbuf(env_key: &str) -> Option<PathBuf> {
    env_to_str(env_key).and_then(|x| PathBuf::from_str(&x[..]).ok())
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
