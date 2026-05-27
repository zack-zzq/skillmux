use crate::{
    api::ApiClient,
    config::{target_skill_dir, Config},
    git_backend::gix,
    installer, skill_manifest,
    sources::{clawhub::ClawHubSource, github, SkillSource},
    storage::SkillStorage,
};
use anyhow::{anyhow, Result};
use directories::BaseDirs;
use std::{
    fs,
    io::{self, Write},
};

pub fn parse_skill_identifier(v: &str) -> (Option<String>, String, Option<String>) {
    let (s, ver) = v
        .split_once('@')
        .map(|(a, b)| (a.to_string(), Some(b.to_string())))
        .unwrap_or((v.to_string(), None));
    if let Some((src, slug)) = s.split_once(':') {
        (Some(src.into()), slug.into(), ver)
    } else {
        (None, s, ver)
    }
}
#[allow(clippy::too_many_arguments)]
pub fn run(
    api: &ApiClient,
    claw: &ClawHubSource,
    source: &str,
    cfg: &Config,
    skill: &str,
    version: Option<String>,
    ref_name: Option<String>,
    subdir: Option<String>,
    as_name: Option<String>,
    yes: bool,
    force: bool,
    json: bool,
) -> Result<()> {
    if let Some(mut gh) = github::parse(skill) {
        /* unchanged github */
        gh.r#ref = ref_name.unwrap_or_else(|| "HEAD".into());
        gh.subdir = subdir.clone();
        if !yes {
            print!("Install third-party GitHub skill from {} ? [y/N]: ", gh.url);
            io::stdout().flush()?;
            let mut ans = String::new();
            io::stdin().read_line(&mut ans)?;
            if ans.trim().to_lowercase() != "y" {
                return Err(anyhow!("aborted"));
            }
        }
        let cache_base = BaseDirs::new()
            .unwrap()
            .cache_dir()
            .join("kdskillhub/github")
            .join(github::cache_key(&gh.owner, &gh.repo, &gh.r#ref));
        let sync = gix::sync(&gh.url, &cache_base.join("repo"), Some(&gh.r#ref))?;
        let src_root = gh
            .subdir
            .as_deref()
            .map(|s| sync.repo_dir.join(s))
            .unwrap_or(sync.repo_dir.clone());
        github::validate_skill_root(&src_root)?;
        let manifest =
            skill_manifest::parse_skill_md(&fs::read_to_string(src_root.join("SKILL.md"))?)?;
        let install_name = as_name.unwrap_or(manifest.name);
        let gh_desc = github::repo_description(&gh.owner, &gh.repo);
        for t in &cfg.install.targets {
            let st = SkillStorage::new(target_skill_dir(t));
            if st.installed(&install_name) && !force {
                continue;
            }
            installer::install_dir(&src_root, &st.skill_path(&install_name))?;
            st.save_info(&install_name,&serde_json::json!({"name":install_name,"version":sync.commit,"slug":install_name,"source":{"type":"github","description":gh_desc,"owner":gh.owner,"repo":gh.repo,"ref":gh.r#ref,"commit":sync.commit}}))?;
        }
        if json {
            println!("{}", serde_json::json!({"name":install_name}));
        }
        return Ok(());
    }
    let (pref, slug, ver2) = parse_skill_identifier(skill);
    let src = pref.unwrap_or_else(|| source.into());
    let ver = version.or(ver2);
    let s: &dyn SkillSource = if src == "clawhub" { claw } else { api };
    s.pre_install_check(&slug)?;
    let resolved = s.resolve(&slug)?;
    let v = ver.or(resolved.version).unwrap_or_else(|| "latest".into());
    let zip = s.download(&slug, Some(&v))?;
    let tmp = tempfile::tempdir()?;
    zip::ZipArchive::new(std::io::Cursor::new(zip))?.extract(tmp.path())?;
    for t in &cfg.install.targets {
        let st = SkillStorage::new(target_skill_dir(t));
        installer::install_dir_copy(tmp.path(), &st.skill_path(&slug))?;
        st.save_info(&slug,&serde_json::json!({"source":{"type":src,"description":resolved.description},"name":resolved.name,"version":v,"slug":slug,"canonical_url":resolved.canonical_url}))?;
    }
    Ok(())
}
