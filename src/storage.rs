use anyhow::Result;
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct SkillStorage {
    pub base: PathBuf,
}
impl SkillStorage {
    pub fn new(base: PathBuf) -> Self {
        let _ = fs::create_dir_all(&base);
        Self { base }
    }
    pub fn skill_path(&self, n: &str) -> PathBuf {
        self.base.join(n)
    }
    pub fn installed(&self, n: &str) -> bool {
        self.skill_path(n).join("SKILL.md").exists()
    }
    pub fn remove(&self, n: &str) -> Result<()> {
        let p = self.skill_path(n);
        if p.exists() {
            fs::remove_dir_all(p)?;
        }
        Ok(())
    }
    pub fn save_info(&self, n: &str, v: &Value) -> Result<()> {
        let d = self.skill_path(n).join(".skillhub");
        fs::create_dir_all(&d)?;
        fs::write(d.join("info.json"), serde_json::to_vec_pretty(v)?)?;
        Ok(())
    }
    pub fn list(&self) -> Result<Vec<String>> {
        Ok(fs::read_dir(&self.base)?
            .flatten()
            .filter(|e| e.path().is_dir())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect())
    }
}

pub fn find_skill_root(root: &Path) -> PathBuf {
    let mut cur = root.to_path_buf();
    loop {
        if cur.join("SKILL.md").exists() {
            return cur;
        }
        let subs: Vec<_> = fs::read_dir(&cur)
            .ok()
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| e.path().is_dir())
            .filter(|e| {
                let n = e.file_name().to_string_lossy().to_string();
                !n.starts_with('.') && n != "__MACOSX"
            })
            .collect();
        if subs.len() != 1 {
            return cur;
        }
        cur = subs[0].path();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn root_logic() {
        let d = tempfile::tempdir().unwrap();
        let a = d.path().join("a");
        let b = a.join("b");
        std::fs::create_dir_all(&b).unwrap();
        std::fs::write(b.join("SKILL.md"), "x").unwrap();
        assert_eq!(find_skill_root(&a), b);
    }
}
