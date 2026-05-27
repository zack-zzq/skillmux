use crate::{
    api::ApiClient,
    commands::install,
    config::{target_skill_dir, Config},
    sources::clawhub::ClawHubSource,
    storage::SkillStorage,
};
use anyhow::Result;

pub fn run(
    api: &ApiClient,
    claw: &ClawHubSource,
    cfg: &Config,
    skill: Option<String>,
    all: bool,
    ref_name: Option<String>,
) -> Result<()> {
    let mut targets = vec![];
    for t in &cfg.install.targets {
        let s = SkillStorage::new(target_skill_dir(t));
        for n in s.list().unwrap_or_default() {
            if !(all || skill.as_deref() == Some(&n)) {
                continue;
            }
            if let Some(info) = s.load_info(&n) {
                targets.push((
                    n,
                    info.source
                        .get("type")
                        .and_then(|v| v.as_str())
                        .or_else(|| info.source.as_str())
                        .unwrap_or("kingdee")
                        .to_string(),
                ));
            }
        }
    }
    targets.sort();
    targets.dedup();
    for (name, src) in targets {
        install::run(
            api,
            claw,
            &src,
            cfg,
            &format!("{}:{}", src, name),
            None,
            ref_name.clone(),
            None,
            None,
            true,
            true,
            false,
        )?;
    }
    Ok(())
}
