//! wind CLI — entry point

mod app;
mod cli;
mod config;
mod errors;
mod platform;
mod tools;
mod windlocal;
mod workspace;

use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = cli::Cli::parse();
    app::run(cli)
}
