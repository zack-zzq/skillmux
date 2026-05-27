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
    path::{Path, PathBuf},
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
    let skill_root = resolve_skill_root(tmp.path())?;
    for t in &cfg.install.targets {
        let st = SkillStorage::new(target_skill_dir(t));
        let dst = st.skill_path(&slug);
        installer::install_dir_copy(&skill_root, &dst)?;
        if !dst.join("SKILL.md").exists() {
            if dst.exists() {
                fs::remove_dir_all(&dst)?;
            }
            return Err(anyhow!(
                "install verification failed: SKILL.md missing in {}",
                dst.display()
            ));
        }
        st.save_info(&slug,&serde_json::json!({"source":{"type":src,"description":resolved.description},"name":resolved.name,"version":v,"slug":slug,"canonical_url":resolved.canonical_url}))?;
        if !json {
            println!("Installed {slug} -> {t}");
        }
    }
    Ok(())
}

fn resolve_skill_root(extract_root: &Path) -> Result<PathBuf> {
    if extract_root.join("SKILL.md").exists() {
        return Ok(extract_root.to_path_buf());
    }
    let top_dirs: Vec<_> = fs::read_dir(extract_root)?
        .flatten()
        .filter(|e| e.path().is_dir())
        .collect();
    if top_dirs.len() == 1 {
        let p = top_dirs[0].path();
        if p.join("SKILL.md").exists() {
            return Ok(p);
        }
    }
    let mut candidates = Vec::new();
    for entry in walkdir::WalkDir::new(extract_root).into_iter().flatten() {
        if entry.file_type().is_file() && entry.file_name() == "SKILL.md" {
            if let Some(parent) = entry.path().parent() {
                candidates.push(parent.to_path_buf());
            }
        }
    }
    candidates.sort();
    candidates.dedup();
    match candidates.len() {
        1 => Ok(candidates.remove(0)),
        0 => Err(anyhow!("SKILL.md not found in downloaded archive")),
        _ => Err(anyhow!(
            "multiple SKILL.md candidates found in downloaded archive"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_skill_root;
    use std::{fs, io::Write};

    #[test]
    fn resolve_github_style_single_top_dir() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo_root = tmp.path().join("demo-main");
        fs::create_dir_all(&repo_root).expect("mkdir");
        fs::write(repo_root.join("SKILL.md"), "# demo").expect("write skill");
        fs::write(repo_root.join("README.md"), "readme").expect("write readme");

        let out = tempfile::tempdir().expect("out");
        let zip_path = out.path().join("skill.zip");
        let f = fs::File::create(&zip_path).expect("zip create");
        let mut zip = zip::ZipWriter::new(f);
        let options = zip::write::SimpleFileOptions::default();
        zip.add_directory("demo-main/", options).expect("add dir");
        zip.start_file("demo-main/SKILL.md", options)
            .expect("start");
        zip.write_all(b"# demo").expect("write");
        zip.finish().expect("finish");

        let extract = tempfile::tempdir().expect("extract");
        let bytes = fs::read(&zip_path).expect("read zip");
        zip::ZipArchive::new(std::io::Cursor::new(bytes))
            .expect("archive")
            .extract(extract.path())
            .expect("extract");

        let root = resolve_skill_root(extract.path()).expect("root");
        assert!(root.join("SKILL.md").exists());
        assert!(!extract.path().join("SKILL.md").exists());
        assert_eq!(root.file_name().and_then(|s| s.to_str()), Some("demo-main"));
    }
}
