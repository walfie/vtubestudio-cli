mod args;

use crate::args::{
    Args, ArtmeshesCommand, Command, Config, HotkeysCommand, ModelsCommand, ParamsCommand,
};

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

        Command::State => {
            let resp = client.send(&ApiStateRequest {}).await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }

        Command::Folders => {
            let resp = client.send(&VtsFolderInfoRequest {}).await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }

        Command::Stats => {
            let resp = client.send(&StatisticsRequest {}).await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }

        Command::SceneColors => {
            let resp = client.send(&SceneColorOverlayInfoRequest {}).await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }

        Command::FaceFound => {
            let resp = client.send(&FaceFoundRequest {}).await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }

        Command::Params(command) => {
            handle_params_command(&mut client, command).await?;
        }

        Command::Hotkeys(command) => {
            handle_hotkeys_command(&mut client, command).await?;
        }

        Command::Artmeshes(command) => {
            handle_artmeshes_command(&mut client, command).await?;
        }

        Command::Models(command) => {
            handle_models_command(&mut client, command).await?;
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

async fn handle_params_command(client: &mut Client, command: ParamsCommand) -> Result<()> {
    use ParamsCommand::*;

    match command {
        Create(req) => {
            let resp = client
                .send(&ParameterCreationRequest {
                    parameter_name: req.name,
                    explanation: req.explanation,
                    min: req.min,
                    max: req.max,
                    default_value: req.default,
                })
                .await?;

            println!("{}", serde_json::to_string_pretty(&resp)?);
        }

        Get { name } => {
            let resp = client.send(&ParameterValueRequest { name }).await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }

        ListLive2D => {
            let resp = client.send(&Live2DParameterListRequest {}).await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }

        ListInputs => {
            let resp = client.send(&InputParameterListRequest {}).await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }

        Delete { name } => {
            let resp = client
                .send(&ParameterDeletionRequest {
                    parameter_name: name,
                })
                .await?;

            println!("{}", serde_json::to_string_pretty(&resp)?);
        }

        Inject(req) => {
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
    }

    Ok(())
}

async fn handle_hotkeys_command(client: &mut Client, command: HotkeysCommand) -> Result<()> {
    use HotkeysCommand::*;

    match command {
        List { model_id } => {
            let resp = client
                .send(&HotkeysInCurrentModelRequest { model_id })
                .await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }

        Trigger(req) => {
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
    }

    Ok(())
}

async fn handle_artmeshes_command(client: &mut Client, command: ArtmeshesCommand) -> Result<()> {
    use ArtmeshesCommand::*;

    match command {
        List => {
            let resp = client.send(&ArtMeshListRequest {}).await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }

        Tint(req) => {
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
    }

    Ok(())
}

async fn handle_models_command(client: &mut Client, command: ModelsCommand) -> Result<()> {
    use ModelsCommand::*;

    match command {
        List => {
            let resp = client.send(&AvailableModelsRequest {}).await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }

        Current => {
            let resp = client.send(&CurrentModelRequest {}).await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }

        Load { id, name } => {
            let model_id = if let Some(id) = id {
                id
            } else if let Some(name) = name {
                let resp = client.send(&AvailableModelsRequest {}).await?;

                resp.available_models
                    .into_iter()
                    .find(|model| model.model_name == name)
                    .with_context(|| format!("no model found with name `{}`", name))?
                    .model_id
            } else {
                bail!("either `id` or `name` must be specified");
            };

            let resp = client.send(&ModelLoadRequest { model_id }).await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }

        Move(req) => {
            let resp = client
                .send(&MoveModelRequest {
                    time_in_seconds: req.duration.as_millis() as f64 / 1000.0,
                    values_are_relative_to_model: req.relative,
                    position_x: req.x,
                    position_y: req.y,
                    rotation: req.rotation,
                    size: req.size,
                })
                .await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }
    }

    Ok(())
}
