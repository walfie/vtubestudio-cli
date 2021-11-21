mod args;

use crate::args::{Args, Command, Config};

use anyhow::{bail, Context, Result};
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
        Command::Param { id } => {
            let resp = client.send(&ParameterValueRequest { name: id }).await?;

            println!("{}", serde_json::to_string_pretty(&resp)?);
        }
        Command::DeleteParam { id } => {
            let resp = client
                .send(&ParameterDeletionRequest { parameter_name: id })
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
        Command::Hotkeys { model_id } => {
            let resp = client
                .send(&HotkeysInCurrentModelRequest { model_id })
                .await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }
        Command::TriggerHotkey(req) => {
            let hotkey_id = if let Some(id) = req.id {
                id
            } else if let Some(name) = req.name {
                let resp = client
                    .send(&HotkeysInCurrentModelRequest { model_id: None })
                    .await?;

                resp.available_hotkeys
                    .into_iter()
                    .find(|hotkey| hotkey.name == name)
                    .with_context(|| format!("no hotkey found with name `{}`", name))?
                    .hotkey_id
            } else {
                bail!("either `id` or `name` must be specified");
            };

            let resp = client.send(&HotkeyTriggerRequest { hotkey_id }).await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }
        Command::Artmeshes => {
            let resp = client.send(&ArtMeshListRequest {}).await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }
        Command::Tint(req) => {
            let resp = client
                .send(&ColorTintRequest {
                    color_tint: ColorTint {
                        color_r: req.color.r,
                        color_g: req.color.g,
                        color_b: req.color.b,
                        color_a: req.color.a,
                        mix_with_scene_lighting_color: req.mix_scene_lighting,
                        jeb_: req.rainbow,
                    },
                    art_mesh_matcher: ArtMeshMatcher {
                        tint_all: req.all,
                        art_mesh_number: req.art_mesh_number,
                        name_exact: req.name_exact,
                        name_contains: req.name_contains,
                        tag_exact: req.tag_exact,
                        tag_contains: req.tag_contains,
                    },
                })
                .await?;

            println!("{}", serde_json::to_string_pretty(&resp)?);

            if resp.matched_art_meshes > 0 {
                info!(
                    duration = ?req.duration,
                    "Tint request successful. Adding delay before exiting..."
                );

                tokio::time::sleep(req.duration).await;
            }
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
