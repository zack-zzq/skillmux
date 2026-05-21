use crate::{
    cli::ConfigCmd,
    config::{parse_targets, Config},
};
use anyhow::Result;
pub fn run(cfg: &mut Config, cmd: ConfigCmd) -> Result<()> {
    match cmd {
        ConfigCmd::List => println!("{}", serde_yaml::to_string(cfg)?),
        ConfigCmd::Get { key } => println!("{}", cfg.get(&key).unwrap_or_default()),
        ConfigCmd::Set { key, value } => {
            cfg.set(&key, &value);
            cfg.save()?;
        }
        ConfigCmd::Targets { targets } => {
            cfg.install.targets = parse_targets(&targets);
            cfg.save()?;
        }
    }
    Ok(())
}
