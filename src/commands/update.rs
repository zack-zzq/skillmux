use crate::{
    api::{extract_list, ApiClient},
    commands::install,
    config::{target_skill_dir, Config},
    storage::SkillStorage,
};
use anyhow::{anyhow, Result};

pub fn run(api: &ApiClient, cfg: &Config, skill: Option<String>, all: bool) -> Result<()> {
    if skill.is_none() && !all {
        println!(
            "Nothing to update. Use `kdskillhub update --all` or `kdskillhub update <skill>`."
        );
        return Ok(());
    }
    let mut installed = vec![];
    for t in &cfg.install.targets {
        let s = SkillStorage::new(target_skill_dir(t));
        for n in s.list().unwrap_or_default() {
            if let Some(info) = s.load_info(&n) {
                installed.push(info);
            }
        }
    }
    installed.sort_by(|a, b| a.name.cmp(&b.name));
    installed.dedup_by(|a, b| a.name == b.name);

    let targets: Vec<_> = if all {
        installed
    } else {
        let name = skill.unwrap();
        installed
            .into_iter()
            .filter(|s| s.name == name)
            .collect::<Vec<_>>()
    };

    if targets.is_empty() {
        return Err(anyhow!("no installed skills matched your update request"));
    }

    let mut changed = 0usize;
    let mut skipped = 0usize;
    for local in targets {
        let data = api.search(Some(local.name.clone()), 1, 20)?;
        let remote = extract_list(&data)?
            .into_iter()
            .find(|s| s.name == local.name)
            .ok_or_else(|| anyhow!("skill `{}` not found on server", local.name))?;
        let latest = remote
            .current_version
            .or(remote.version)
            .ok_or_else(|| anyhow!("skill `{}` has no version on server", local.name))?;
        if latest == local.version {
            println!("Up-to-date: {} v{}", local.name, local.version);
            skipped += 1;
            continue;
        }
        install::run(api, cfg, &local.name, Some(latest.clone()), true, false)?;
        println!("Updated: {} {} -> {}", local.name, local.version, latest);
        changed += 1;
    }
    println!("Update summary: {changed} updated, {skipped} already up-to-date.");
    Ok(())
}
