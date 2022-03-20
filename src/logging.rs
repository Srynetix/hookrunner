use std::future::Future;

use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

use crate::config::Config;

pub struct TracingSetup;

impl TracingSetup {
    pub async fn with_setup<Func, Fut>(config: Config, func: Func) -> color_eyre::Result<()>
    where
        Fut: Future<Output = color_eyre::Result<()>> + Send + 'static,
        Func: FnOnce(Config) -> Fut + Send,
    {
        configure_log_var();

        if let Some(telemetry_url) = config.telemetry_url() {
            let tracer = opentelemetry_jaeger::new_pipeline()
                .with_agent_endpoint(&telemetry_url)
                .install_batch(opentelemetry::runtime::Tokio)?;
            let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

            Registry::default()
                .with(EnvFilter::from_default_env())
                .with(
                    HierarchicalLayer::new(2)
                        .with_targets(true)
                        .with_bracketed_fields(true),
                )
                .with(ErrorLayer::default())
                .with(telemetry)
                .init();

            tokio::spawn(func(config)).await??;

            opentelemetry::global::shutdown_tracer_provider();
        } else {
            Registry::default()
                .with(EnvFilter::from_default_env())
                .with(
                    HierarchicalLayer::new(2)
                        .with_targets(true)
                        .with_bracketed_fields(true),
                )
                .with(ErrorLayer::default())
                .init();

            tokio::spawn(func(config)).await??;
        };

        Ok(())
    }
}

fn configure_log_var() {
    if std::env::var("RUST_LOG")
        .ok()
        .filter(|s| !s.is_empty())
        .is_none()
    {
        std::env::set_var("RUST_LOG", "tower_http=trace,hookrunner=trace,info");
    }
}
