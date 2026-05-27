use anyhow::{anyhow, Result};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Clone)]
pub struct SyncResult {
    pub repo_dir: PathBuf,
    pub commit: String,
}

pub fn sync(url: &str, repo_dir: &Path, r#ref: Option<&str>) -> Result<SyncResult> {
    if !repo_dir.exists() {
        std::fs::create_dir_all(repo_dir.parent().unwrap())?;
        let st = Command::new("gix")
            .args(["clone", url, repo_dir.to_str().unwrap()])
            .status()?;
        if !st.success() {
            return Err(anyhow!("gix clone failed"));
        }
    } else {
        let st = Command::new("gix")
            .current_dir(repo_dir)
            .args(["fetch"])
            .status()?;
        if !st.success() {
            return Err(anyhow!("gix fetch failed"));
        }
    }
    if let Some(r) = r#ref {
        let st = Command::new("gix")
            .current_dir(repo_dir)
            .args(["checkout", r])
            .status()?;
        if !st.success() {
            return Err(anyhow!("gix checkout failed"));
        }
    }
    let out = Command::new("gix")
        .current_dir(repo_dir)
        .args(["rev-parse", "HEAD"])
        .output()?;
    Ok(SyncResult {
        repo_dir: repo_dir.to_path_buf(),
        commit: String::from_utf8_lossy(&out.stdout).trim().to_string(),
    })
}
