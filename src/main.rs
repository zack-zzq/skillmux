mod api;
mod cli;
mod commands;
mod config;
mod storage;

use anyhow::Result;

fn main() -> Result<()> {
    cli::run()
}
