//! wind CLI — entry point

mod app;
mod cli;
mod config;
mod errors;
mod extract;
mod platform;
mod tools;
mod windlocal;
mod workspace;

use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    // Initialize tracing with graceful fallback for non-terminal environments
    let _ = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .try_init();

    // Run the CLI; any errors are handled by app::run which exits with proper codes
    if let Err(e) = app::run(cli::build()) {
        eprintln!("windcli error: {}", e);
        std::process::exit(1);
    }
}
