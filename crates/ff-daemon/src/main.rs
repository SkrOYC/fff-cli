//! ff-daemon - background daemon for ff.

mod lifecycle;

use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use tracing::{error, info};

use lifecycle::DaemonLifecycle;

#[derive(Parser, Debug)]
#[command(name = "ff-daemon", about = "Background daemon for ff")]
struct Args {
    #[arg(long)]
    root: PathBuf,

    #[arg(long, default_value = "300")]
    idle_timeout_secs: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("ff-daemon starting for root: {}", args.root.display());

    let config = ff_common::config::load();
    let idle_timeout = if args.idle_timeout_secs > 0 {
        Duration::from_secs(args.idle_timeout_secs)
    } else {
        config.daemon.idle_timeout
    };

    let lifecycle = DaemonLifecycle::new(args.root.clone(), idle_timeout);

    lifecycle.check_and_clean_stale()?;
    lifecycle.create_pid_file()?;
    let listener = lifecycle.create_socket()?;

    let mut shutdown_rx = lifecycle.shutdown_receiver();

    let idle_check = {
        let shutdown_tx_clone = lifecycle.shutdown_receiver();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));
            let mut rx = shutdown_tx_clone;
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // Idle check would go here in full implementation
                    }
                    _ = rx.changed() => {
                        break;
                    }
                }
            }
        })
    };

    let signal_task = tokio::spawn(async move {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("received SIGINT, shutting down");
            }
            _ = async {
                #[cfg(unix)]
                {
                    let mut sigterm = tokio::signal::unix::signal(
                        tokio::signal::unix::SignalKind::terminate(),
                    ).expect("failed to register SIGTERM handler");
                    sigterm.recv().await;
                    info!("received SIGTERM, shutting down");
                }
                #[cfg(not(unix))]
                {
                    std::future::pending::<()>().await;
                }
            } => {}
        }
        lifecycle.request_shutdown();
    });

    info!(
        "ff-daemon ready, listening on {}",
        listener
            .local_addr()
            .map(|a| a
                .as_pathname()
                .map(|p| p.display().to_string())
                .unwrap_or_default())
            .unwrap_or_default()
    );

    tokio::select! {
        _ = shutdown_rx.changed() => {
            info!("shutdown signal received");
        }
        result = accept_connections(listener) => {
            if let Err(e) = result {
                error!("listener error: {e}");
            }
        }
    }

    signal_task.abort();
    idle_check.abort();

    info!("ff-daemon shutting down");
    Ok(())
}

async fn accept_connections(_listener: tokio::net::UnixListener) -> Result<()> {
    std::future::pending::<()>().await;
    Ok(())
}
