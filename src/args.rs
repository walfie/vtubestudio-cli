use anyhow::{Context, Error, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use structopt::StructOpt;
use vtubestudio::data::{EnumString, FadeMode};

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
    /// Actions related to items.
    #[structopt(alias = "item")]
    Items(ItemsCommand),
    /// Actions related to events.
    #[structopt(alias = "event")]
    Events(EventsCommand),
}

impl Command {
    pub fn is_event_subscription(&self) -> bool {
        matches!(self, Self::Events(_))
    }
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
    /// Whether to use `add` mode instead of `set` mode.
    #[structopt(long)]
    pub add: bool,
}

#[derive(StructOpt, Debug, Clone)]
pub enum HotkeysCommand {
    /// List the available hotkeys for a model or Live2D item.
    List {
        /// Model ID.
        #[structopt(long)]
        model_id: Option<String>,
        /// Live2D item file name.
        #[structopt(long)]
        live2d_file: Option<String>,
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
    /// Trigger hotkey for this item instance ID.
    #[structopt(long)]
    pub item: Option<String>,
}

#[derive(StructOpt, Debug, Clone)]
pub enum ArtmeshesCommand {
    /// List art meshes in the current model.
    List,
    /// Tint matching art meshes.
    Tint(Tint),
    /// Trigger art mesh selection.
    Select {
        /// Text shown over the art mesh selection list.
        #[structopt(long)]
        set_text: Option<String>,
        /// Text shown when the user presses the `?` button.
        #[structopt(long)]
        set_help: Option<String>,
        /// Number of meshes that should be selected.
        #[structopt(long)]
        count: Option<i32>,
        /// Preselect these meshes.
        #[structopt(long)]
        preselect: Vec<String>,
    },
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

#[derive(StructOpt, Debug, Clone)]
pub enum ItemsCommand {
    /// List items.
    List {
        /// Include available spots.
        #[structopt(long)]
        spots: bool,
        /// Include available instances.
        #[structopt(long)]
        instances: bool,
        /// Include available file names.
        #[structopt(long)]
        files: bool,
        /// Only include specific file name.
        #[structopt(long)]
        with_file_name: Option<String>,
        /// Only include specific instance ID.
        #[structopt(long)]
        with_instance_id: Option<String>,
    },
    /// Load item into scene.
    Load(ItemLoadCommand),
    /// Unload item from scene.
    Unload(ItemUnloadCommand),
    /// Move item.
    Move(ItemMoveCommand),
    /// Set item animation properties.
    Animation(ItemAnimationCommand),
}

#[derive(StructOpt, Debug, Clone)]
pub struct ItemLoadCommand {
    /// File name. E.g., `some_item_name.jpg`.
    pub file_name: String,
    /// X position.
    #[structopt(short, default_value = "0")]
    pub x: f64,
    /// Y position.
    #[structopt(short, default_value = "0")]
    pub y: f64,
    #[structopt(long, default_value = "0.32")]
    pub size: f64,
    /// Rotation, in degrees.
    #[structopt(long, default_value = "0")]
    pub rotation: i32,
    /// Fade time, in seconds. Should be between `0` and `2`.
    #[structopt(long, default_value = "0")]
    pub fade_time: f64,
    /// Item order. If the order is taken, VTube Studio will automatically try to find the
    /// next available order, unless `fail_if_order_taken` is `true`.
    #[structopt(long)]
    pub order: Option<i32>,
    /// Set to `true` to fail with an `ItemOrderAlreadyTaken` error if the desired `order`
    /// is already taken.
    #[structopt(long)]
    pub fail_if_order_taken: bool,
    /// Smoothing, between `0` and `1`.
    #[structopt(long, default_value = "0")]
    pub smoothing: f64,
    /// Whether the item is censored.
    #[structopt(long)]
    pub censored: bool,
    /// Whether the item is flipped.
    #[structopt(long)]
    pub flipped: bool,
    /// Whether the item is locked.
    #[structopt(long)]
    pub locked: bool,
}

#[derive(StructOpt, Debug, Clone)]
pub struct ItemUnloadCommand {
    /// Unload all items in the scene.
    #[structopt(long)]
    pub all: bool,
    /// Whether to unload all items loaded by this plugin.
    #[structopt(long)]
    pub from_this_plugin: bool,
    /// Whether to allow unloading items that have been loaded by the user or other
    /// plugins.
    #[structopt(long)]
    pub from_other_plugins: bool,
    /// Request specific instance IDs to be unloaded.
    #[structopt(long)]
    pub id: Vec<String>,
    /// Request specific file names to be unloaded.
    #[structopt(long)]
    pub file: Vec<String>,
}

#[derive(StructOpt, Debug, Clone)]
pub struct ItemMoveCommand {
    pub id: String,
    #[structopt(long, parse(try_from_str = parse_duration::parse))]
    pub duration: Duration,
    #[structopt(
        long,
        parse(from_str = parse_fade_mode),
        default_value = "linear",
        possible_values = FADE_MODES
    )]
    pub fade_mode: EnumString<FadeMode>,
    #[structopt(short)]
    pub x: Option<i32>,
    #[structopt(short)]
    pub y: Option<i32>,
    #[structopt(long)]
    pub size: Option<f64>,
    #[structopt(long)]
    pub rotation: Option<i32>,
    #[structopt(long)]
    pub order: Option<i32>,
    #[structopt(long)]
    pub set_flip: bool,
    #[structopt(long)]
    pub flip: bool,
    #[structopt(long)]
    pub user_can_stop: bool,
}

fn parse_fade_mode(value: &str) -> EnumString<FadeMode> {
    EnumString::<FadeMode>::new_from_str(value.to_owned())
}

const FADE_MODES: &'static [&'static str] = &[
    "linear",
    "easeIn",
    "easeOut",
    "easeBoth",
    "overshoot",
    "zip",
];

#[derive(StructOpt, Debug, Clone)]
pub struct ItemAnimationCommand {
    /// Item instance ID.
    pub item_instance_id: String,
    #[structopt(long)]
    /// Frame rate for animated items, clamped between `0.1` and `120`.
    pub framerate: Option<f64>,
    /// Jump to a specific frame, zero-indexed.
    ///
    /// May return an error if the frame index is invalid, or if the item type does not
    /// support animation.
    #[structopt(long)]
    pub frame: Option<i32>,
    /// Brightness.
    #[structopt(long)]
    pub brightness: Option<f64>,
    /// Opacity.
    #[structopt(long)]
    pub opacity: Option<f64>,
    /// List of frame indices that the animation will automatically stop playing on.
    #[structopt(long, conflicts_with = "reset-stop-frames")]
    pub stop_frame: Vec<i32>,
    /// Unset auto-stop-frames.
    #[structopt(long)]
    pub reset_stop_frames: bool,
    /// Play the animation.
    #[structopt(long, conflicts_with = "stop")]
    pub play: bool,
    /// Stop the animation.
    #[structopt(long, conflicts_with = "play")]
    pub stop: bool,
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

#[derive(StructOpt, Debug, Clone)]
pub enum EventsCommand {
    /// Test events.
    Test {
        /// Test message.
        message: String,
    },

    /// Model loaded.
    ModelLoaded {
        /// Optional model IDs to filter for.
        #[structopt(long)]
        model_id: Vec<String>,
    },

    /// Tracking status changed (face/hand tracking found or lost).
    TrackingStatusChanged {},

    /// Background changed.
    BackgroundChanged {},

    /// Model config changed.
    ///
    /// Triggered every time the user manually changes the the settings/config of the currently
    /// loaded VTube Studio model.
    ModelConfigChanged {},

    /// Model moved.
    ModelMoved {},

    /// Model outline.
    ModelOutline {
        /// Whether to draw the outline.
        #[structopt(long)]
        draw: bool,
    },
}
