use anyhow::{Context, Error, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(global_setting = structopt::clap::AppSettings::AllowNegativeNumbers)]
pub struct Args {
    /// Overwrite path to config file.
    #[structopt(env = "VTS_CONFIG", long)]
    pub config_file: Option<PathBuf>,
    /// Avoid pretty-printing JSON.
    #[structopt(long)]
    pub compact: bool,
    #[structopt(subcommand)]
    pub command: Command,
}

#[derive(Clone, Debug, Serialize, Deserialize, StructOpt)]
pub struct Config {
    #[structopt(short, long, default_value = "localhost")]
    pub host: String,
    #[structopt(short, long, default_value = "8001")]
    pub port: u16,
    #[structopt(long, env = "VTS_TOKEN", hide_env_values = true)]
    pub token: Option<String>,
    #[structopt(long, default_value = "VTube Studio CLI")]
    pub plugin_name: String,
    #[structopt(long, default_value = "Walfie")]
    pub plugin_developer: String,
}

#[derive(StructOpt, Debug, Clone)]
pub enum Command {
    /// Actions related to configuration of this program.
    Config(ConfigCommand),
    /// Get the current state of the API.
    State,
    /// VTube Studio statistics.
    Stats,
    /// Get a list of VTube Studio folders.
    Folders,
    /// Actions related to parameters.
    #[structopt(alias = "param")]
    Params(ParamsCommand),
    /// Actions related to hotkeys.
    #[structopt(alias = "hotkey")]
    Hotkeys(HotkeysCommand),
    /// Actions related to artmeshes.
    #[structopt(alias = "artmesh")]
    Artmeshes(ArtmeshesCommand),
    /// Actions related to models.
    #[structopt(alias = "model")]
    Models(ModelsCommand),
    /// Scene color overlay info.
    SceneColors,
    /// Checking if face is currently found by tracker.
    FaceFound,
    /// Actions related to expressions.
    #[structopt(alias = "expression")]
    Expressions(ExpressionsCommand),
    /// Actions related to NDI Config.
    Ndi(NdiCommand),
    /// Actions related to physics.
    Physics(PhysicsCommand),
}

#[derive(StructOpt, Debug, Clone)]
pub enum ConfigCommand {
    /// Requests permissions from VTube Studio to initialize config file.
    Init(Config),
    /// Shows the contents of config file.
    Show,
    /// Outputs the config file path.
    Path,
}

#[derive(StructOpt, Debug, Clone)]
pub enum ParamsCommand {
    /// Get the value of a parameter.
    Get {
        /// Name of the parameter.
        name: String,
    },
    /// Create a custom parameter.
    Create(CreateParam),
    /// Temporarily set the value for a custom parameter.
    ///
    /// VTube Studio will reset this value if it hasn't been updated at least once per second.
    Inject(InjectParam),
    /// Delete a custom parameter.
    Delete {
        /// Name of the parameter.
        name: String,
    },
    /// Get the value for all input parameters in the current model.
    ListInputs,
    /// Get the value for all Live2D parameters in the current model.
    #[structopt(name = "list-live2d")]
    ListLive2D,
}

#[derive(StructOpt, Debug, Clone)]
pub struct CreateParam {
    pub name: String,
    #[structopt(long, default_value = "0")]
    pub default: f64,
    #[structopt(long, default_value = "0")]
    pub min: f64,
    #[structopt(long, default_value = "100")]
    pub max: f64,
    #[structopt(long)]
    pub explanation: Option<String>,
}

#[derive(StructOpt, Debug, Clone)]
pub struct InjectParam {
    pub id: String,
    pub value: f64,
    #[structopt(long)]
    pub weight: Option<f64>,
    #[structopt(long)]
    pub face_found: bool,
}

#[derive(StructOpt, Debug, Clone)]
pub enum HotkeysCommand {
    /// List the available hotkeys for a model.
    List {
        /// Model ID.
        #[structopt(long)]
        model_id: Option<String>,
    },
    /// Trigger hotkey by ID or name.
    Trigger(TriggerHotkey),
}

#[derive(StructOpt, Debug, Clone)]
pub struct TriggerHotkey {
    /// Hotkey ID to trigger.
    #[structopt(conflicts_with = "name")]
    pub id: Option<String>,
    /// Find and trigger the first hotkey with this name, if it exists.
    #[structopt(long, conflicts_with = "id")]
    pub name: Option<String>,
}

#[derive(StructOpt, Debug, Clone)]
pub enum ArtmeshesCommand {
    /// List art meshes in the current model.
    List,
    /// Tint matching art meshes.
    Tint(Tint),
}

#[derive(StructOpt, Debug, Clone)]
pub enum ExpressionsCommand {
    /// List art meshes in the current model.
    List {
        /// Whether to return additional details.
        #[structopt(long)]
        details: bool,
        /// Return only the state of this expression file.
        file: Option<String>,
    },
    /// Activate an expression.
    Activate { file: String },
    /// Deactivate an expression.
    Deactivate { file: String },
}

#[derive(StructOpt, Debug, Clone)]
pub struct Tint {
    /// Enable `jeb_` (rainbow) mode.
    #[structopt(long, alias = "jeb_")]
    pub rainbow: bool,
    /// Mix with scene lighting color value (between 0 and 1).
    #[structopt(long)]
    pub mix_scene_lighting: Option<f64>,
    /// Hex color code with optional alpha.
    #[structopt(long, default_value = "#ffffff")]
    pub color: HexColor,
    /// Match all art meshes.
    #[structopt(long)]
    pub all: bool,
    #[structopt(long)]
    pub art_mesh_number: Vec<i32>,
    #[structopt(long)]
    pub name_exact: Vec<String>,
    #[structopt(long)]
    pub name_contains: Vec<String>,
    #[structopt(long)]
    pub tag_exact: Vec<String>,
    #[structopt(long)]
    pub tag_contains: Vec<String>,
    /// How long the tint should last for (e.g., `5s`, `1m30s`).
    ///
    /// This is needed because VTube Studio resets the tint when the plugin disconnects, and unless
    /// we add a delay, this CLI tool exits immediately after submitting the request.
    #[structopt(long, parse(try_from_str = parse_duration::parse))]
    pub duration: Duration,
}

#[derive(Debug, Clone)]
pub struct HexColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl FromStr for HexColor {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self> {
        let ([r, g, b], a) = read_color::rgb_maybe_a(&mut value.trim_start_matches('#').chars())
            .with_context(|| format!("could not parse string `{}` as a hex color value", value))?;

        Ok(HexColor {
            r,
            g,
            b,
            a: a.unwrap_or(255),
        })
    }
}

#[derive(StructOpt, Debug, Clone)]
pub enum ModelsCommand {
    /// List available models.
    List,
    /// Get current model.
    Current,
    /// Load a model by ID or name.
    Load {
        /// Model ID to load.
        #[structopt(conflicts_with = "name")]
        id: Option<String>,
        /// Load the first model with this name, if it exists.
        #[structopt(long, conflicts_with = "id")]
        name: Option<String>,
    },
    /// Move the current model.
    Move(MoveModel),
}

#[derive(StructOpt, Debug, Clone)]
pub struct MoveModel {
    /// How long the movement animation should take.
    #[structopt(long, default_value = "0s", parse(try_from_str = parse_duration::parse))]
    pub duration: Duration,
    /// Whether the movement is relative to the current model position.
    #[structopt(long)]
    pub relative: bool,
    /// Horizontal position. -1 for left edge, 1 for right edge.
    #[structopt(long)]
    pub x: Option<f64>,
    /// Vertical position. -1 for bottom edge, 1 for top edge.
    #[structopt(long)]
    pub y: Option<f64>,
    /// Rotation in degrees, between -360 and 360.
    #[structopt(long)]
    pub rotation: Option<f64>,
    /// Size, between -100 and 100.
    #[structopt(long)]
    pub size: Option<f64>,
}

#[derive(StructOpt, Debug, Clone)]
pub enum NdiCommand {
    /// Shows the current NDI config.
    GetConfig,
    /// Set NDI config.
    SetConfig(NdiSetConfig),
}

#[derive(StructOpt, Debug, Clone)]
pub struct NdiSetConfig {
    /// Whether NDI should be active.
    #[structopt(long, takes_value = true)]
    pub active: Option<bool>,
    /// Whether NDI 5 should be used.
    #[structopt(long)]
    pub use_ndi5: Option<bool>,
    /// Whether a custom resolution should be used.
    ///
    /// Setting this to `true` means the NDI stream will no longer have
    /// the same resolution as the VTube Studio window, but instead use
    /// the custom resolution set via the UI or the `custom_width`
    /// fields of this request.
    #[structopt(long, takes_value = true)]
    pub use_custom_resolution: Option<bool>,
    /// Custom NDI width if `use_custom_resolution` is specified.
    ///
    /// Must be a multiple of 16 and be between `256` and `8192`.
    #[structopt(long)]
    pub width: Option<i32>,
    /// Custom NDI height if `use_custom_resolution` is specified.
    ///
    /// Must be a multiple of 8 and be between `256` and `8192`.
    #[structopt(long)]
    pub height: Option<i32>,
}

#[derive(StructOpt, Debug, Clone)]
pub enum PhysicsCommand {
    /// Gets physics settings of the current model.
    Get,
    /// Sets physics settings.
    Set(SetPhysicsCommand),
}

#[derive(StructOpt, Debug, Clone)]
pub enum SetPhysicsCommand {
    /// Set the base value.
    Base(SetBasePhysicsConfig),
    /// Set the multipler value.
    Multiplier(SetMultiplierPhysicsConfig),
}

impl SetPhysicsCommand {
    pub fn kind(&self) -> &StrengthOrWind {
        match self {
            Self::Base(conf) => &conf.kind,
            Self::Multiplier(conf) => &conf.kind,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum StrengthOrWind {
    Strength,
    Wind,
}

impl StrengthOrWind {
    fn variants() -> &'static [&'static str] {
        &["strength", "wind"]
    }
}

impl FromStr for StrengthOrWind {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            "strength" => Self::Strength,
            "wind" => Self::Wind,
            other => anyhow::bail!(
                "Unknown value `{}`. Should be either `strength` or `wind`.",
                other
            ),
        })
    }
}

#[derive(StructOpt, Debug, Clone)]
pub struct SetBasePhysicsConfig {
    /// Type of physics (strength or wind).
    #[structopt(possible_values = &StrengthOrWind::variants())]
    pub kind: StrengthOrWind,
    /// Base value. Should be between 0 and 100.
    pub value: u8,
    /// How long to override the value for.
    ///
    /// Should be between 0.5s and 5s.
    #[structopt(long, default_value = "500ms", parse(try_from_str = parse_duration::parse))]
    pub duration: Duration,
}

#[derive(StructOpt, Debug, Clone)]
pub struct SetMultiplierPhysicsConfig {
    /// Type of physics (strength or wind).
    #[structopt(possible_values = &StrengthOrWind::variants())]
    pub kind: StrengthOrWind,
    /// Multiplier value. Should be between 0 and 2.
    pub value: f64,
    /// Group ID.
    #[structopt(long)]
    pub id: String,
    /// How long to override the value for.
    ///
    /// Should be between 0.5s and 5s.
    #[structopt(long, default_value = "500ms", parse(try_from_str = parse_duration::parse))]
    pub duration: Duration,
}
