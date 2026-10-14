use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};

/// Default URL for Home Assistant host
pub const DEFAULT_HA_URL: &str = "http://localhost:8123";

/// Default empty token placeholder (users should supply via CLI or HASS_TOKEN)
pub const DEFAULT_HA_TOKEN: &str = "";

#[derive(Parser, Debug, Clone)]
#[command(
    name = "ha-tui",
    author = "Beckett",
    version = "0.1.0",
    about = "Terminal dashboard for Home Assistant built with Ratatui"
)]
pub struct CliArgs {
    /// Home Assistant base URL (default: http://localhost:8123 or HASS_URL)
    #[arg(short, long, env = "HASS_URL", default_value = DEFAULT_HA_URL)]
    pub url: String,

    /// Home Assistant Long-Lived Access Token
    #[arg(short, long, env = "HASS_TOKEN")]
    pub token: Option<String>,

    /// Polling interval in seconds (default: 5)
    #[arg(short = 'i', long, default_value = "5")]
    pub poll_interval: u64,

    /// Path to .env file to load
    #[arg(short = 'e', long)]
    pub env_file: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub url: String,
    pub token: String,
    pub poll_interval: u64,
}

impl Config {
    pub fn load() -> Result<Self, anyhow::Error> {
        let args = CliArgs::parse();

        // 1. URL: Always defaults to hardcoded Tailscale IP http://100.91.201.64:8123
        let clean_url = args.url.trim_end_matches('/').to_string();

        // 2. Token: CLI -> Environment -> .env files -> Hardcoded token default
        let mut token = args
            .token
            .or_else(|| std::env::var("HASS_TOKEN").ok())
            .or_else(|| std::env::var("HA_TOKEN").ok());

        if token.is_none() {
            let mut candidates: Vec<PathBuf> = Vec::new();
            if let Some(custom) = &args.env_file {
                candidates.push(custom.clone());
            }

            candidates.push(PathBuf::from(".env"));
            candidates.push(PathBuf::from("../.env"));
            if let Ok(home) = std::env::var("HOME") {
                candidates.push(PathBuf::from(home).join(".config/ha-tui/config.env"));
            }

            for path in candidates {
                if path.exists() {
                    let _ = read_env_file(&path, &mut token);
                    if token.is_some() {
                        break;
                    }
                }
            }
        }

        let final_token = token.unwrap_or_else(|| DEFAULT_HA_TOKEN.to_string());

        Ok(Config {
            url: clean_url,
            token: final_token,
            poll_interval: args.poll_interval.max(1),
        })
    }
}

fn read_env_file(path: &Path, token: &mut Option<String>) -> std::io::Result<()> {
    let content = fs::read_to_string(path)?;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let key = k.trim();
            let val = v.trim().trim_matches('"').trim_matches('\'').to_string();
            if (key == "HASS_TOKEN" || key == "HA_TOKEN") && token.is_none() && !val.is_empty() {
                *token = Some(val);
            }
        }
    }
    Ok(())
}
