use crate::{
    cli::ConfigCmd,
    config::{install_check_path, parse_targets_args, Config, ALL_TARGETS},
};
use anyhow::Result;
use comfy_table::{presets::UTF8_FULL, Attribute, Cell, Table};
pub fn run(cfg: &mut Config, cmd: ConfigCmd) -> Result<()> {
    match cmd {
        ConfigCmd::List => {
            let mut table = Table::new();
            table.load_preset(UTF8_FULL);
            table.set_header(vec!["Key", "Value"]);
            table.add_row(vec![
                Cell::new("api.endpoint"),
                Cell::new(cfg.api.endpoint.clone()),
            ]);
            table.add_row(vec![
                Cell::new("api.timeout"),
                Cell::new(cfg.api.timeout.to_string()),
            ]);
            table.add_row(vec![
                Cell::new("api.token"),
                Cell::new(cfg.api.token.clone().unwrap_or_default()),
            ]);
            table.add_row(vec![
                Cell::new("source.default"),
                Cell::new(cfg.source.default.clone()),
            ]);
            table.add_row(vec![
                Cell::new("install.targets"),
                Cell::new(cfg.install.targets.join(", ")),
            ]);
            println!("{table}");
        }
        ConfigCmd::Get { key } => println!("{}", cfg.get(&key).unwrap_or_default()),
        ConfigCmd::Set { key, value } => {
            cfg.set(&key, &value)?;
            cfg.save()?;
        }
        ConfigCmd::Targets { targets } => {
            if targets.is_empty() {
                let mut table = Table::new();
                table.load_preset(UTF8_FULL);
                table.set_header(vec![
                    Cell::new("Target").add_attribute(Attribute::Bold),
                    Cell::new("Selected").add_attribute(Attribute::Bold),
                    Cell::new("Install Path").add_attribute(Attribute::Bold),
                ]);
                for t in ALL_TARGETS {
                    table.add_row(vec![
                        Cell::new(*t),
                        Cell::new(if cfg.install.targets.iter().any(|x| x == t) {
                            "yes"
                        } else {
                            "no"
                        }),
                        Cell::new(install_check_path(t).display().to_string()),
                    ]);
                }
                println!("{table}");
            } else {
                cfg.install.targets = parse_targets_args(&targets)?;
                cfg.save()?;
                println!(
                    "Updated install targets: {}",
                    cfg.install.targets.join(", ")
                );
            }
        }
    }
    Ok(())
}
