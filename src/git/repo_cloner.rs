use std::path::{Path, PathBuf};

use crate::{config::Config, service::ServiceHandler};

use super::{GitBackend, GitError, RefType, RepositoryPath};

pub struct RepoCloner;

impl RepoCloner {
    #[tracing::instrument]
    pub async fn create_or_update_using_config(
        config: &Config,
        services: &ServiceHandler,
        backend: GitBackend,
        repo_path: &RepositoryPath,
        reference: RefType,
    ) -> Result<(), GitError> {
        let repository_target_dir =
            Self::get_repository_target_dir(config, &repo_path.full_name(), repo_path.name());

        Self::create_or_update_in_directory(
            services,
            backend,
            repo_path,
            reference,
            &repository_target_dir,
        )
        .await
    }

    #[tracing::instrument]
    pub async fn create_or_update_in_directory(
        services: &ServiceHandler,
        backend: GitBackend,
        repo_full_name: &RepositoryPath,
        reference: RefType,
        working_dir: &Path,
    ) -> Result<(), GitError> {
        let root_url = backend.root_url();
        let repo_full_name = repo_full_name.full_name();
        let ref_name = reference.to_string();
        let repo_url: String = format!("{root_url}/{repo_full_name}");

        if !working_dir.exists() {
            // Get folder name
            let folder_name = working_dir.file_stem().unwrap().to_string_lossy();

            // Clone the repository
            services
                .git()
                .clone_repository(
                    working_dir.parent().unwrap(),
                    &ref_name,
                    &repo_url,
                    &folder_name,
                )
                .await?;
        } else {
            services.git().fetch(working_dir).await?;
            services.git().checkout(working_dir, &ref_name).await?;
            services.git().pull(working_dir).await?;
        }

        Ok(())
    }

    fn get_working_dir(config: &Config) -> PathBuf {
        if let Some(d) = config.working_dir() {
            PathBuf::from(d)
        } else {
            std::env::current_dir().unwrap()
        }
    }

    fn get_repository_target_dir(
        config: &Config,
        repo_full_name: &str,
        folder_name: &str,
    ) -> PathBuf {
        if let Some(value) = config.repo_mapping().get(repo_full_name) {
            PathBuf::from(value)
        } else {
            Self::get_working_dir(config).join(folder_name)
        }
    }
}
