mod api;
mod cli;
mod commands;
mod config;
mod git_backend;
mod installer;
mod skill_manifest;
mod sources;
mod storage;

use anyhow::Result;

fn main() -> Result<()> {
    cli::run()
}
