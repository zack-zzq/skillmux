use anyhow::{anyhow, Result};
use git2::{AutotagOption, FetchOptions, Oid, Repository};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct SyncResult {
    pub repo_dir: PathBuf,
    pub commit: String,
}

#[derive(thiserror::Error, Debug)]
pub enum GitSyncError {
    #[error("network failure while accessing repository: {0}")]
    Network(String),
    #[error("repository not found or inaccessible: {0}")]
    RepoNotFound(String),
    #[error("requested ref not found: {0}")]
    RefNotFound(String),
    #[error("checkout failed for ref {0}: {1}")]
    CheckoutFailed(String, String),
    #[error("local repository cache is corrupted: {0}")]
    CacheCorrupted(String),
    #[error("git operation failed: {0}")]
    Other(String),
}

pub fn sync(url: &str, repo_dir: &Path, r#ref: Option<&str>) -> Result<SyncResult> {
    let repo = if repo_dir.exists() {
        open_or_reclone(repo_dir, url)?
    } else {
        clone_repo(url, repo_dir)?
    };

    fetch_repo(&repo, url)?;

    let resolved = resolve_ref(&repo, r#ref.unwrap_or("HEAD"))?;
    checkout_commit(&repo, resolved, r#ref.unwrap_or("HEAD"))?;

    Ok(SyncResult {
        repo_dir: repo_dir.to_path_buf(),
        commit: resolved.to_string(),
    })
}

fn open_or_reclone(repo_dir: &Path, url: &str) -> Result<Repository> {
    match Repository::open(repo_dir) {
        Ok(repo) => Ok(repo),
        Err(e) => {
            std::fs::remove_dir_all(repo_dir).map_err(|ioe| {
                anyhow!(GitSyncError::CacheCorrupted(format!(
                    "{e}; failed to remove corrupted cache: {ioe}"
                )))
            })?;
            clone_repo(url, repo_dir)
        }
    }
}

fn clone_repo(url: &str, repo_dir: &Path) -> Result<Repository> {
    if let Some(parent) = repo_dir.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Repository::clone(url, repo_dir).map_err(map_git2_err)
}

fn fetch_repo(repo: &Repository, url: &str) -> Result<()> {
    let mut remote = match repo.find_remote("origin") {
        Ok(r) => r,
        Err(_) => repo.remote_anonymous(url).map_err(map_git2_err)?,
    };
    let mut fo = FetchOptions::new();
    fo.download_tags(AutotagOption::All);
    remote
        .fetch(
            &[
                "+refs/heads/*:refs/remotes/origin/*",
                "+refs/tags/*:refs/tags/*",
            ],
            Some(&mut fo),
            None,
        )
        .map_err(map_git2_err)?;
    Ok(())
}

fn resolve_ref(repo: &Repository, ref_name: &str) -> Result<Oid> {
    if ref_name == "HEAD" {
        return repo
            .head()
            .and_then(|h| h.peel_to_commit())
            .map(|c| c.id())
            .map_err(map_git2_err);
    }
    if let Ok(oid) = Oid::from_str(ref_name) {
        if repo.find_object(oid, None).is_ok() {
            return Ok(oid);
        }
    }
    let candidates = [
        ref_name.to_string(),
        format!("refs/heads/{ref_name}"),
        format!("refs/tags/{ref_name}"),
        format!("refs/remotes/origin/{ref_name}"),
    ];
    for cand in candidates {
        if let Ok(obj) = repo.revparse_single(&cand) {
            return Ok(obj.id());
        }
    }
    Err(anyhow!(GitSyncError::RefNotFound(ref_name.to_string())))
}

fn checkout_commit(repo: &Repository, oid: Oid, ref_name: &str) -> Result<()> {
    let obj = repo.find_object(oid, None).map_err(map_git2_err)?;
    repo.checkout_tree(&obj, None).map_err(|e| {
        anyhow!(GitSyncError::CheckoutFailed(
            ref_name.to_string(),
            e.message().to_string()
        ))
    })?;
    repo.set_head_detached(oid).map_err(|e| {
        anyhow!(GitSyncError::CheckoutFailed(
            ref_name.to_string(),
            e.message().to_string()
        ))
    })?;
    Ok(())
}

fn map_git2_err(err: git2::Error) -> anyhow::Error {
    let msg = err.message().to_string();
    let out = if msg.contains("not found") || msg.contains("Repository not found") {
        GitSyncError::RepoNotFound(msg)
    } else if msg.contains("Could not resolve host")
        || msg.contains("timed out")
        || msg.contains("connection")
        || msg.contains("TLS")
    {
        GitSyncError::Network(msg)
    } else {
        GitSyncError::Other(msg)
    };
    anyhow!(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_ref_supports_commit_and_branch() {
        let td = tempfile::tempdir().expect("tempdir");
        let repo = Repository::init(td.path()).expect("init");
        let sig = git2::Signature::now("t", "t@t.com").expect("sig");
        std::fs::write(td.path().join("a.txt"), "a").expect("write");
        let mut idx = repo.index().expect("index");
        idx.add_path(Path::new("a.txt")).expect("add");
        idx.write().expect("idx write");
        let tree_id = idx.write_tree().expect("tree");
        let tree = repo.find_tree(tree_id).expect("find tree");
        let c = repo
            .commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
            .expect("commit");

        let by_head = resolve_ref(&repo, "HEAD").expect("head");
        let by_main = resolve_ref(&repo, "master").expect("master");
        let by_oid = resolve_ref(&repo, &c.to_string()).expect("oid");
        assert_eq!(by_head, c);
        assert_eq!(by_main, c);
        assert_eq!(by_oid, c);
    }
}
