//! Main Entrypoint for darkkirb.de
//!
//! This crate serves as the main entry point for darkkirb.de and mostly just sets up all of the components of the website
#![forbid(unsafe_code)]

use anyhow::Result;
use tracing::{error, info};
use tracing_subscriber::{
    fmt::time::ChronoUtc, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt,
    Layer,
};

mod gemini;

#[tracing::instrument]
async fn run() -> Result<()> {
    info!("Starting up...");
    gemini::run_gemini().await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv()?;

    #[cfg(not(debug_assertions))]
    {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::from_default_env().and_then(
                    tracing_subscriber::fmt::layer()
                        .with_timer(ChronoUtc::rfc3339())
                        .json(),
                ),
            )
            .with(sentry::integrations::tracing::layer())
            .try_init()?;
    }
    #[cfg(debug_assertions)]
    {
        tracing_subscriber::registry()
            .with(sentry::integrations::tracing::layer())
            .with(
                tracing_subscriber::EnvFilter::from_default_env().and_then(
                    tracing_subscriber::fmt::layer()
                        .with_timer(ChronoUtc::rfc3339())
                        .pretty(),
                ),
            )
            .try_init()?;
    }

    let _guard = sentry::init((
        std::env::var("SENTRY_DSN").ok(),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            attach_stacktrace: true,
            debug: true,
            ..Default::default()
        }
        .add_integration(sentry::integrations::backtrace::AttachStacktraceIntegration::new())
        .add_integration(sentry::integrations::backtrace::ProcessStacktraceIntegration::new())
        .add_integration(sentry::integrations::contexts::ContextIntegration::new())
        .add_integration(sentry::integrations::debug_images::DebugImagesIntegration::new())
        .add_integration(sentry::integrations::panic::PanicIntegration::new()),
    ));

    let result = run().await;

    if let Err(ref e) = result {
        sentry::integrations::anyhow::capture_anyhow(e);
        error!("{:?}", e);
    }

    Ok(())
}
