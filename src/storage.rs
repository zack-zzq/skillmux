use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs, path::PathBuf};

pub struct SkillStorage {
    pub base: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledSkill {
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub source: Value,
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
        let p = self.skill_path(n);
        p.join("SKILL.md").exists() || p.join("skill.md").exists()
    }
    pub fn remove(&self, n: &str) -> Result<()> {
        let p = self.skill_path(n);
        if p.exists() {
            if p.is_symlink() {
                fs::remove_file(p)?;
            } else {
                fs::remove_dir_all(p)?;
            }
        }
        Ok(())
    }
    pub fn save_info(&self, n: &str, v: &Value) -> Result<()> {
        let d = self.skill_path(n).join(".skillhub");
        fs::create_dir_all(&d)?;
        fs::write(d.join("info.json"), serde_json::to_vec_pretty(v)?)?;
        Ok(())
    }
    pub fn load_info(&self, n: &str) -> Option<InstalledSkill> {
        let p = self.skill_path(n).join(".skillhub/info.json");
        let b = fs::read(p).ok()?;
        serde_json::from_slice(&b).ok()
    }
    pub fn list(&self) -> Result<Vec<String>> {
        Ok(fs::read_dir(&self.base)?
            .flatten()
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect())
    }
}
