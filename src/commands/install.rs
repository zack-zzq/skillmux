use crate::{api::{extract_list, ApiClient}, config::{install_check_path, target_skill_dir, Config}, git_backend::gix, installer, skill_manifest, sources::github, storage::SkillStorage};
use anyhow::{anyhow, Result};
use directories::BaseDirs;
use std::{fs, io::{self, Write}, path::PathBuf};

pub fn parse_skill_identifier(v: &str) -> (String, Option<String>) { v.split_once('@').map(|(a,b)|(a.into(),Some(b.into()))).unwrap_or((v.into(),None)) }

#[allow(clippy::too_many_arguments)]
pub fn run(api: &ApiClient, cfg: &Config, skill: &str, version: Option<String>, ref_name: Option<String>, subdir: Option<String>, as_name: Option<String>, yes: bool, force: bool, json: bool) -> Result<()> {
    if let Some(mut gh) = github::parse(skill) {
        gh.r#ref = ref_name.unwrap_or_else(|| "HEAD".into());
        gh.subdir = subdir.clone();
        if !yes {
            print!("Install third-party GitHub skill from {} ? [y/N]: ", gh.url);
            io::stdout().flush()?;
            let mut ans = String::new(); io::stdin().read_line(&mut ans)?;
            if ans.trim().to_lowercase() != "y" { return Err(anyhow!("aborted")); }
        }
        let cache_base = BaseDirs::new().unwrap().cache_dir().join("kdskillhub/github").join(github::cache_key(&gh.owner,&gh.repo,&gh.r#ref));
        let sync = gix::sync(&gh.url, &cache_base.join("repo"), Some(&gh.r#ref))?;
        let src_root = gh.subdir.as_deref().map(|s| sync.repo_dir.join(s)).unwrap_or(sync.repo_dir.clone());
        github::validate_skill_root(&src_root)?;
        let manifest = skill_manifest::parse_skill_md(&fs::read_to_string(src_root.join("SKILL.md"))?)?;
        let install_name = as_name.unwrap_or(manifest.name);
        for t in &cfg.install.targets {
            if !install_check_path(t).exists() { continue; }
            let st = SkillStorage::new(target_skill_dir(t));
            if st.installed(&install_name) && !force { continue; }
            installer::install_dir(&src_root, &st.skill_path(&install_name))?;
            st.save_info(&install_name, &serde_json::json!({"name":install_name,"version":sync.commit,"target":t,"source":{"type":"github","owner":gh.owner,"repo":gh.repo,"url":gh.url,"ref":gh.r#ref,"subdir":gh.subdir,"commit":sync.commit,"backend":"gix","pinned":gh.r#ref.len()==40}}))?;
        }
        if json { println!("{}", serde_json::json!({"name":install_name})); }
        return Ok(());
    }

    let (name, ver2) = parse_skill_identifier(skill);
    let ver = version.or(ver2);
    let data = api.search(Some(name.clone()), 1, 20)?;
    let one = extract_list(&data)?.into_iter().find(|s| s.name==name).ok_or_else(|| anyhow!("not found"))?;
    let v = ver.or(one.current_version).or(one.version).ok_or_else(|| anyhow!("no version"))?;
    let zip = api.download(one.id, &v)?;
    let tmp = tempfile::tempdir()?;
    zip::ZipArchive::new(std::io::Cursor::new(zip))?.extract(tmp.path())?;
    let src_root = tmp.path();
    for t in &cfg.install.targets {
        let st = SkillStorage::new(target_skill_dir(t));
        installer::install_dir(src_root, &st.skill_path(&name))?;
        st.save_info(&name, &serde_json::json!({"name":name,"version":v,"target":t,"source":{"type":"kingdee","id":one.id}}))?;
    }
    Ok(())
}
