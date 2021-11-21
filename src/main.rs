use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;
use tracing::{error, info};
use vtubestudio::data::*;
use vtubestudio::Client;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = Args::from_args();

    tracing_subscriber::fmt::fmt().init();

    let config_path = match args.config_file {
        Some(path) => path,
        None => xdg::BaseDirectories::with_prefix("vtubestudio-cli")?
            .place_config_file("config.json")
            .context("Failed to find config path")?,
    };

    let mut conf: Config = if let Command::Init(conf) = &args.command {
        conf.clone()
    } else {
        let json_str = std::fs::read_to_string(&config_path).with_context(|| {
            let bin = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("vts"));
            format!(
                "failed to load config file from {:?} (try running `{:?} init` to create the file)",
                &config_path, bin
            )
        })?;

        serde_json::from_str(&json_str).context("failed to parse JSON from config file")?
    };

    let (mut client, mut new_tokens) = Client::builder()
        .auth_token(conf.token.clone())
        .authentication(
            conf.plugin_name.clone(),
            conf.plugin_developer.clone(),
            None,
        )
        .build_tungstenite();

    match args.command {
        Command::Init(..) => {
            info!("Requesting plugin permissions. Please accept the permissions pop-up in the VTube Studio app.");
            client.send(&StatisticsRequest {}).await?;
        }
        Command::Stats => {
            let stats = client.send(&StatisticsRequest {}).await?;
            println!("{}", serde_json::to_string_pretty(&stats)?);
        }
    };

    drop(client);

    if let Some(new_token) = new_tokens.next().await {
        conf.token = Some(new_token);
        if let Err(e) = std::fs::write(&config_path, serde_json::to_string_pretty(&conf)?) {
            error!(?config_path, "Failed to write config file");
            anyhow::bail!(e);
        } else {
            info!(?config_path, "Wrote authentication token to config file");
        }
    }

    Ok(())
}

#[derive(StructOpt)]
struct Args {
    /// Overwrite path to config file.
    ///
    /// If this is unspecified and `$XDG_CONFIG_HOME` is unset, the default config path is
    /// `~/.config/vtubestudio-cli/config.json`, otherwise
    /// `$XDG_CONFIG_HOME/vtubestudio-cli/config.json`.
    #[structopt(env)]
    config_file: Option<PathBuf>,
    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt)]
enum Command {
    /// Request permissions from VTube Studio to initialize config file.
    Init(Config),
    /// VTube Studio statistics.
    Stats,
}

#[derive(Clone, Debug, Serialize, Deserialize, StructOpt)]
struct Config {
    #[structopt(short, long, default_value = "localhost")]
    host: String,
    #[structopt(short, long, default_value = "8001")]
    port: u16,
    #[structopt(long, env, hide_env_values = true)]
    token: Option<String>,
    #[structopt(long, default_value = "VTube Studio CLI")]
    plugin_name: String,
    #[structopt(long, default_value = "Walfie")]
    plugin_developer: String,
}
