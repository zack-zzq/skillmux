use crate::{api::ApiClient, commands, config::Config, sources::clawhub::ClawHubSource};
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "skillmux",
    version,
    about = "A fast multi-source, multi-target skill manager."
)]
pub struct Cli {
    #[arg(short, long)]
    pub config: Option<String>,
    #[arg(short = 'a', long)]
    pub api: Option<String>,
    #[arg(long)]
    pub token: Option<String>,
    #[arg(long)]
    pub source: Option<String>,
    #[command(subcommand)]
    pub command: Cmd,
}
#[derive(Subcommand)]
pub enum Cmd {
    Search {
        keyword: Option<String>,
        #[arg(long, default_value_t = 20)]
        limit: u32,
        #[arg(long, default_value_t = 1)]
        page: u32,
        #[arg(long)]
        json: bool,
    },
    Install {
        skill: String,
        #[arg(long)]
        version: Option<String>,
        #[arg(long = "ref")]
        ref_name: Option<String>,
        #[arg(long)]
        subdir: Option<String>,
        #[arg(long = "as")]
        as_name: Option<String>,
        #[arg(short = 'y', long)]
        yes: bool,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        json: bool,
    },
    List {
        #[arg(long)]
        json: bool,
    },
    Update {
        skill: Option<String>,
        #[arg(long)]
        all: bool,
        #[arg(long = "ref")]
        ref_name: Option<String>,
    },
    #[command(visible_alias = "uninstall")]
    Remove {
        skill: String,
        #[arg(long)]
        purge: bool,
    },
    Config {
        #[command(subcommand)]
        sub: ConfigCmd,
    },
}
#[derive(Subcommand)]
pub enum ConfigCmd {
    List,
    Get { key: String },
    Set { key: String, value: String },
    Targets { targets: String },
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let mut cfg = Config::load(cli.config.as_deref())?;
    let endpoint = cli.api.clone().unwrap_or_else(|| cfg.api.endpoint.clone());
    let token = cfg.resolve_token(cli.token.clone());
    let api = ApiClient::new(endpoint, cfg.api.timeout, token)?;
    let claw = ClawHubSource::new(None, cfg.api.timeout)?;
    let source = cli.source.unwrap_or_else(|| cfg.source.default.clone());
    ensure_builtin_skills(&cfg)?;
    commands::dispatch(cli.command, &mut cfg, &api, &claw, &source)
}

fn ensure_builtin_skills(cfg: &Config) -> Result<()> {
    use std::fs;
    let marker = directories::BaseDirs::new()
        .unwrap()
        .home_dir()
        .join(".config/skillhub/.skillmux_bootstrap_done");
    if marker.exists() {
        return Ok(());
    }
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("skills");
    if src.exists() {
        for t in &cfg.install.targets {
            let dst = crate::config::target_skill_dir(t);
            fs::create_dir_all(&dst)?;
            for e in fs::read_dir(&src)? {
                let e = e?;
                let to = dst.join(e.file_name());
                if !to.exists() {
                    crate::installer::install_dir_copy(&e.path(), &to)?;
                }
            }
        }
    }
    if let Some(p) = marker.parent() {
        fs::create_dir_all(p)?;
    }
    fs::write(marker, b"ok")?;
    Ok(())
}
