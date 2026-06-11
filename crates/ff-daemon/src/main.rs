//! ff-daemon - background daemon for ff.

mod lifecycle;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use tracing::info;

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

    let lifecycle = Arc::new(tokio::sync::Mutex::new(DaemonLifecycle::new(
        args.root.clone(),
        idle_timeout,
    )));

    {
        let lc = lifecycle.lock().await;
        lc.check_and_clean_stale()?;
        lc.create_pid_file()?;
        let _listener = lc.create_socket()?;
    }

    let mut shutdown_rx = {
        let lc = lifecycle.lock().await;
        lc.shutdown_receiver()
    };

    let idle_lifecycle = lifecycle.clone();
    let idle_check = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        let mut rx = {
            let lc = idle_lifecycle.lock().await;
            lc.shutdown_receiver()
        };
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let lc = idle_lifecycle.lock().await;
                    if lc.is_idle_timed_out() {
                        info!("idle timeout expired, shutting down");
                        lc.request_shutdown();
                        break;
                    }
                }
                _ = rx.changed() => {
                    break;
                }
            }
        }
    });

    let signal_lifecycle = lifecycle.clone();
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
        let lc = signal_lifecycle.lock().await;
        lc.request_shutdown();
    });

    info!("ff-daemon ready");

    tokio::select! {
        _ = shutdown_rx.changed() => {
            info!("shutdown signal received");
        }
    }

    signal_task.abort();
    idle_check.abort();

    info!("ff-daemon shutting down");
    Ok(())
}
