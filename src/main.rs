mod args;

use crate::args::{
    Args, ArtmeshesCommand, Command, Config, ConfigCommand, ExpressionsCommand, HotkeysCommand,
    ModelsCommand, NdiCommand, ParamsCommand, PhysicsCommand, SetPhysicsCommand, StrengthOrWind,
};

use anyhow::{bail, Context, Result};
use once_cell::sync::OnceCell;
use serde::Serialize;
use std::path::PathBuf;
use structopt::StructOpt;
use tracing::{error, info};
use vtubestudio::data::*;
use vtubestudio::Client;

static JSON_COMPACT: OnceCell<bool> = OnceCell::new();

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = Args::from_args();
    let _ = JSON_COMPACT.set(args.compact);

    tracing_subscriber::fmt::fmt().init();

    let config_path = match args.config_file {
        Some(path) => path,
        None => {
            let mut path =
                directories::ProjectDirs::from("com.github", "walfie", "vtubestudio-cli")
                    .context("failed to get base directory")?
                    .config_dir()
                    .to_path_buf();

            path.push("config.json");
            path
        }
    };

    let mut conf: Config = if let Command::Config(ConfigCommand::Init(conf)) = &args.command {
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
        Command::Config(command) => {
            use ConfigCommand::*;

            match command {
                Init(..) => {
                    info!("Requesting plugin permissions. Please accept the permissions pop-up in the VTube Studio app.");
                    client.send(&StatisticsRequest {}).await?;
                }
                Show => {
                    print(&conf)?;
                }
                Path => {
                    println!("{:?}", config_path);
                }
            }
        }

        Command::State => {
            print(&client.send(&ApiStateRequest {}).await?)?;
        }

        Command::Folders => {
            print(&client.send(&VtsFolderInfoRequest {}).await?)?;
        }

        Command::Stats => {
            print(&client.send(&StatisticsRequest {}).await?)?;
        }

        Command::SceneColors => {
            print(&client.send(&SceneColorOverlayInfoRequest {}).await?)?;
        }

        Command::FaceFound => {
            print(&client.send(&FaceFoundRequest {}).await?)?;
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

        Command::Expressions(command) => {
            handle_expressions_command(&mut client, command).await?;
        }

        Command::Ndi(command) => {
            handle_ndi_command(&mut client, command).await?;
        }

        Command::Physics(command) => {
            handle_physics_command(&mut client, command).await?;
        }
    };

    drop(client);

    if let Some(new_token) = new_tokens.next().await {
        conf.token = Some(new_token);

        let mut base_path = config_path.clone();
        base_path.pop();
        std::fs::create_dir_all(&base_path)
            .with_context(|| format!("Failed to create directory {:?}", base_path))?;

        if let Err(e) = std::fs::write(&config_path, serde_json::to_string_pretty(&conf)?) {
            error!(?config_path, "Failed to write config file");
            anyhow::bail!(e);
        } else {
            info!(?config_path, "Wrote authentication token to config file");
        }
    }

    Ok(())
}

fn print<T: Serialize>(value: &T) -> Result<()> {
    let string = if *JSON_COMPACT.get().unwrap_or(&false) {
        serde_json::to_string(value)?
    } else {
        serde_json::to_string_pretty(value)?
    };

    println!("{}", string);
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

            print(&resp)?;
        }

        Get { name } => {
            print(&client.send(&ParameterValueRequest { name }).await?)?;
        }

        ListLive2D => {
            print(&client.send(&Live2DParameterListRequest {}).await?)?;
        }

        ListInputs => {
            print(&client.send(&InputParameterListRequest {}).await?)?;
        }

        Delete { name } => {
            let resp = client
                .send(&ParameterDeletionRequest {
                    parameter_name: name,
                })
                .await?;

            print(&resp)?;
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

            print(&resp)?;
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
            print(&resp)?;
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
            print(&resp)?;
        }
    }

    Ok(())
}

async fn handle_artmeshes_command(client: &mut Client, command: ArtmeshesCommand) -> Result<()> {
    use ArtmeshesCommand::*;

    match command {
        List => {
            print(&client.send(&ArtMeshListRequest {}).await?)?;
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

            print(&resp)?;

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
            print(&client.send(&AvailableModelsRequest {}).await?)?;
        }

        Current => {
            print(&client.send(&CurrentModelRequest {}).await?)?;
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
            print(&resp)?;
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
            print(&resp)?;
        }
    }

    Ok(())
}

async fn handle_expressions_command(
    client: &mut Client,
    command: ExpressionsCommand,
) -> Result<()> {
    use ExpressionsCommand::*;

    match command {
        List { details, file } => {
            let resp = client
                .send(&ExpressionStateRequest {
                    details,
                    expression_file: file,
                })
                .await?;
            print(&resp)?;
        }

        Activate { file } => {
            let resp = client
                .send(&ExpressionActivationRequest {
                    expression_file: file,
                    active: true,
                })
                .await?;
            print(&resp)?;
        }

        Deactivate { file } => {
            let resp = client
                .send(&ExpressionActivationRequest {
                    expression_file: file,
                    active: false,
                })
                .await?;
            print(&resp)?;
        }
    }

    Ok(())
}

async fn handle_ndi_command(client: &mut Client, command: NdiCommand) -> Result<()> {
    use NdiCommand::*;

    match command {
        GetConfig => {
            let resp = client
                .send(&NdiConfigRequest {
                    set_new_config: false,
                    ..NdiConfigRequest::default()
                })
                .await?;
            print(&resp)?;
        }

        SetConfig(value) => {
            let resp = client
                .send(&NdiConfigRequest {
                    set_new_config: true,
                    ndi_active: value.active,
                    use_ndi5: value.use_ndi5,
                    use_custom_resolution: value.use_custom_resolution,
                    custom_width_ndi: value.width,
                    custom_height_ndi: value.height,
                })
                .await?;
            print(&resp)?;
        }
    }

    Ok(())
}

async fn handle_physics_command(client: &mut Client, command: PhysicsCommand) -> Result<()> {
    use PhysicsCommand::*;

    match command {
        Get => {
            let resp = client.send(&GetCurrentModelPhysicsRequest {}).await?;
            print(&resp)?;
        }

        Set(mut value) => {
            use SetPhysicsCommand::*;

            let mut req = SetCurrentModelPhysicsRequest::default();
            let mut physics = PhysicsOverride::default();

            match &mut value {
                Base(base) => {
                    physics.set_base_value = true;
                    physics.value = base.value as f64;
                    physics.override_seconds = base.duration.as_secs_f64();
                }
                Multiplier(mult) => {
                    std::mem::swap(&mut physics.id, &mut mult.id);
                    physics.value = mult.value;
                    physics.override_seconds = mult.duration.as_secs_f64();
                }
            }

            match value.kind() {
                StrengthOrWind::Strength => {
                    req.strength_overrides = vec![physics];
                }
                StrengthOrWind::Wind => {
                    req.wind_overrides = vec![physics];
                }
            }

            let resp = client.send(&req).await?;
            print(&resp)?;
        }
    }

    Ok(())
}
