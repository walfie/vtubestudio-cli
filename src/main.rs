mod args;

use crate::args::{Args, Command, Config};

use anyhow::{Context, Result};
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
            let resp = client.send(&StatisticsRequest {}).await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }
        Command::CreateParam(req) => {
            let resp = client
                .send(&ParameterCreationRequest {
                    parameter_name: req.id,
                    explanation: req.explanation,
                    min: req.min,
                    max: req.max,
                    default_value: req.default,
                })
                .await?;

            println!("{}", serde_json::to_string_pretty(&resp)?);
        }
        Command::SetParam(req) => {
            let resp = client
                .send(&InjectParameterDataRequest {
                    parameter_values: vec![ParameterValue {
                        id: req.id,
                        value: req.value,
                        weight: req.weight,
                    }],
                })
                .await?;

            println!("{}", serde_json::to_string_pretty(&resp)?);
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
