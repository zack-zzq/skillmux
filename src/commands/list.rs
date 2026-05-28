use crate::{
    config::{target_skill_dir, Config},
    storage::SkillStorage,
};
use anyhow::Result;
use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Cell, ContentArrangement, Table,
};

pub fn run(cfg: &Config, json: bool) -> Result<()> {
    let mut rows = vec![];
    for t in &cfg.install.targets {
        let s = SkillStorage::new(target_skill_dir(t)?);
        for n in s.list()? {
            if let Some(info) = s.load_info(&n) {
                let src = info
                    .source
                    .get("type")
                    .and_then(|v| v.as_str())
                    .or_else(|| info.source.as_str())
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .unwrap_or("legacy");
                let desc = info
                    .source
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                rows.push(serde_json::json!({
                    "target": t,
                    "name": info.name,
                    "source": src,
                    "version": info.version,
                    "description": desc,
                }));
            }
        }
    }
    if json {
        println!("{}", serde_json::to_string_pretty(&rows)?);
    } else {
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic);
        table.set_header(vec!["Target", "Name", "Source", "Version", "Description"]);
        for r in rows {
            let desc = r["description"].as_str().unwrap_or("");
            let clipped = if desc.chars().count() > 80 {
                format!("{}...", desc.chars().take(77).collect::<String>())
            } else {
                desc.to_string()
            };
            table.add_row(vec![
                Cell::new(r["target"].as_str().unwrap_or_default()),
                Cell::new(r["name"].as_str().unwrap_or_default()),
                Cell::new(r["source"].as_str().unwrap_or_default()),
                Cell::new(r["version"].as_str().unwrap_or_default()),
                Cell::new(clipped),
            ]);
        }
        println!("{table}");
    }
    Ok(())
}
