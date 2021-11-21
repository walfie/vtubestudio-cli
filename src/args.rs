use anyhow::{Context, Error, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
pub struct Args {
    /// Overwrite path to config file.
    ///
    /// If this is unspecified and `$XDG_CONFIG_HOME` is unset, the default config path is
    /// `~/.config/vtubestudio-cli/config.json`, otherwise
    /// `$XDG_CONFIG_HOME/vtubestudio-cli/config.json`.
    #[structopt(env)]
    pub config_file: Option<PathBuf>,
    #[structopt(subcommand)]
    pub command: Command,
}

#[derive(Clone, Debug, Serialize, Deserialize, StructOpt)]
pub struct Config {
    #[structopt(short, long, default_value = "localhost")]
    pub host: String,
    #[structopt(short, long, default_value = "8001")]
    pub port: u16,
    #[structopt(long, env, hide_env_values = true)]
    pub token: Option<String>,
    #[structopt(long, default_value = "VTube Studio CLI")]
    pub plugin_name: String,
    #[structopt(long, default_value = "Walfie")]
    pub plugin_developer: String,
}

#[derive(StructOpt, Debug, Clone)]
pub enum Command {
    /// Request permissions from VTube Studio to initialize config file.
    Init(Config),
    /// VTube Studio statistics.
    Stats,
    /// Create a custom parameter.
    CreateParam(CreateParam),
    /// Temporarily set the value for a custom parameter.
    SetParam(SetParam),
    /// List the available hotkeys for a model.
    Hotkeys {
        /// Model ID.
        #[structopt(long)]
        model_id: Option<String>,
    },
    /// Trigger hotkey by ID or name.
    TriggerHotkey(TriggerHotkey),
    /// List art meshes in the current model.
    Artmeshes,
    /// Tint matching art meshes.
    Tint(Tint),
}

#[derive(StructOpt, Debug, Clone)]
pub struct CreateParam {
    pub id: String,
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
pub struct SetParam {
    pub id: String,
    pub value: f64,
    #[structopt(long)]
    pub weight: Option<f64>,
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
