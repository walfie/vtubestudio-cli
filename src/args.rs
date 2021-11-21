use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
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

#[derive(StructOpt)]
pub enum Command {
    /// Request permissions from VTube Studio to initialize config file.
    Init(Config),
    /// VTube Studio statistics.
    Stats,
    /// Create a custom parameter.
    CreateParam(CreateParam),
}

#[derive(StructOpt)]
pub struct CreateParam {
    #[structopt(long)]
    pub param_id: String,
    #[structopt(long, default_value = "0")]
    pub default: f64,
    #[structopt(long, default_value = "0")]
    pub min: f64,
    #[structopt(long, default_value = "100")]
    pub max: f64,
    #[structopt(long)]
    pub explanation: Option<String>,
}
