use crate::{
    api::ApiClient,
    sources::{clawhub::ClawHubSource, SkillSource},
};
use anyhow::Result;
pub fn run(
    api: &ApiClient,
    claw: &ClawHubSource,
    source: &str,
    keyword: Option<String>,
    limit: u32,
    page: u32,
    json: bool,
) -> Result<()> {
    let rows = if source == "clawhub" {
        claw.search(keyword, limit, page)?
    } else {
        crate::sources::SkillSource::search(api, keyword, limit, page)?
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&rows)?);
    } else {
        for s in rows {
            println!(
                "{}\t{}\t{}",
                s.slug,
                s.version.unwrap_or_default(),
                s.description.unwrap_or_default()
            );
        }
    }
    Ok(())
}
