use crate::{
    api::{extract_list, ApiClient},
    config::{install_check_path, target_skill_dir, Config},
    storage::{find_skill_root, SkillStorage},
};
use anyhow::{anyhow, Result};
use std::{fs, io::Cursor};
use zip::ZipArchive;
pub fn parse_skill_identifier(v: &str) -> (String, Option<String>) {
    if let Some((a, b)) = v.split_once('@') {
        (a.to_string(), Some(b.to_string()))
    } else {
        (v.to_string(), None)
    }
}
pub fn run(
    api: &ApiClient,
    cfg: &Config,
    skill: &str,
    version: Option<String>,
    force: bool,
    json: bool,
) -> Result<()> {
    let (name, ver2) = parse_skill_identifier(skill);
    let ver = version.or(ver2);
    let data = api.search(Some(name.clone()), 1, 20)?;
    let skills = extract_list(&data)?;
    let one = skills
        .into_iter()
        .find(|s| s.name == name)
        .ok_or_else(|| anyhow!("not found"))?;
    let v = ver
        .or(one.current_version)
        .or(one.version)
        .ok_or_else(|| anyhow!("no version"))?;
    let zip = api.download(one.id, &v)?;
    let mut installed_targets = 0usize;
    let mut skipped_targets = 0usize;
    for t in &cfg.install.targets {
        if !install_check_path(t).exists() {
            continue;
        }
        let st = SkillStorage::new(target_skill_dir(t));
        if st.installed(&name) && !force {
            skipped_targets += 1;
            continue;
        }
        let dest = st.skill_path(&name);
        if dest.exists() {
            fs::remove_dir_all(&dest)?;
        }
        fs::create_dir_all(&dest)?;
        let mut ar = ZipArchive::new(Cursor::new(zip.clone()))?;
        ar.extract(&dest)?;
        let root = find_skill_root(&dest);
        if root != dest {
            for e in fs::read_dir(&root)? {
                let e = e?;
                fs::rename(e.path(), dest.join(e.file_name()))?;
            }
        }
        st.save_info(
            &name,
            &serde_json::json!({"id":one.id,"name":name,"version":v}),
        )?;
        installed_targets += 1;
    }
    if json {
        println!("{}", serde_json::json!({"name":name,"version":v}));
    } else {
        println!(
            "Installed {name}@{v} to {installed_targets} target(s), skipped {skipped_targets}."
        );
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse() {
        assert_eq!(
            parse_skill_identifier("a@1"),
            ("a".into(), Some("1".into()))
        );
        assert_eq!(parse_skill_identifier("a"), ("a".into(), None));
    }
}
