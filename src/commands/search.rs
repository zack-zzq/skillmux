use crate::api::{extract_list, ApiClient};
use anyhow::Result;
pub fn run(
    api: &ApiClient,
    keyword: Option<String>,
    limit: u32,
    page: u32,
    json: bool,
) -> Result<()> {
    let v = api.search(keyword, page, limit)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&v)?);
    } else {
        for s in extract_list(&v)? {
            println!(
                "{}\t{}",
                s.name,
                s.current_version.or(s.version).unwrap_or_default()
            );
        }
    }
    Ok(())
}
