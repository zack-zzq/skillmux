use anyhow::Result;
use std::{fs, path::Path};

pub fn install_dir(src: &Path, dst: &Path) -> Result<()> {
    install_dir_with_mode(src, dst, true)
}

pub fn install_dir_copy(src: &Path, dst: &Path) -> Result<()> {
    install_dir_with_mode(src, dst, false)
}

fn install_dir_with_mode(src: &Path, dst: &Path, allow_symlink: bool) -> Result<()> {
    if dst.exists() {
        if dst.is_symlink() {
            fs::remove_file(dst)?;
        } else {
            fs::remove_dir_all(dst)?;
        }
    }
    if allow_symlink {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(src, dst)?;
            return Ok(());
        }
        #[cfg(windows)]
        {
            if std::os::windows::fs::symlink_dir(src, dst).is_ok() {
                return Ok(());
            }
        }
    }
    copy_dir(src, dst)
}

fn copy_dir(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in walkdir::WalkDir::new(src).into_iter().flatten() {
        let rel = entry.path().strip_prefix(src)?;
        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}
