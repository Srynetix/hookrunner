use clap::Parser;

use hookrunner::backends::github;
use hookrunner::cmdargs::{Args, ServeCommand, SubCommand};
use hookrunner::config::{Config, ConfigError, ServerConfig};
use hookrunner::git::RepoCloner;
use hookrunner::http::start_server;
use hookrunner::logging::TracingSetup;
use hookrunner::service::ServiceHandler;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> color_eyre::Result<()> {
    dotenv::dotenv().ok();
    color_eyre::install().ok();

    let args = Args::parse();
    let config = build_configuration(&args)?;
    let services = ServiceHandler::new_defaults()?;

    TracingSetup::with_setup(config, |config| async move {
        match args.command {
            SubCommand::Serve(serve_args) => {
                let server_config = build_server_configuration(&serve_args)?;
                start_server(server_config, config, services).await?
            }
            SubCommand::Synchronize(sync_args) => {
                RepoCloner::create_or_update_using_config(
                    &config,
                    &services,
                    sync_args.backend,
                    &sync_args.repository,
                    sync_args.reference,
                )
                .await?;
            }
            SubCommand::Install(install_args) => {
                let client = github::Client::new(install_args.token);
                let repo = install_args.repository;

                client
                    .try_register_webhook(&config, repo.owner(), repo.name(), &install_args.url)
                    .await?;
            }
            SubCommand::Uninstall(install_args) => {
                let client = github::Client::new(install_args.token);
                let repo = install_args.repository;
                client
                    .try_unregister_webhook(&config, repo.owner(), repo.owner(), &install_args.url)
                    .await?;
            }
        }

        Ok(())
    })
    .await
}

fn build_configuration(args: &Args) -> Result<Config, ConfigError> {
    let mut config = Config::from_env();

    if let Some(m) = &args.github_api_url {
        config.set_github_api_url(m.clone());
    }

    if let Some(m) = &args.repo_mapping {
        config.set_repo_mapping(m);
    }

    if let Some(t) = &args.telemetry_url {
        config.set_telemetry_url(t.clone());
    }

    if let Some(w) = &args.working_dir {
        config.set_working_dir(w);
    }

    if let Some(s) = &args.webhook_secret {
        config.set_webhook_secret(s);
    }

    config.validate_configuration().map(|_| config)
}

fn build_server_configuration(args: &ServeCommand) -> Result<ServerConfig, ConfigError> {
    let mut server_config = ServerConfig::from_env()?;

    if let Some(m) = &args.bind_ip {
        server_config.set_bind_ip(*m);
    }

    Ok(server_config)
}
