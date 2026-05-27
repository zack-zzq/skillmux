use std::{env, fs, path::PathBuf};

fn main() {
    let token = env::var("KDSKILLHUB_DEFAULT_TOKEN").unwrap_or_default();
    let key: u8 = 0x5A;
    let obfuscated: Vec<String> = token
        .as_bytes()
        .iter()
        .map(|b| format!("{}", b ^ key))
        .collect();

    let out = format!(
        "pub const TOKEN_KEY: u8 = {key};\npub const TOKEN_DATA: &[u8] = &[{data}];\n",
        key = key,
        data = obfuscated.join(",")
    );

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR missing"));
    fs::write(out_dir.join("generated_token.rs"), out).expect("write generated token failed");

    let skills_root = PathBuf::from("skills");
    println!("cargo:rerun-if-changed={}", skills_root.display());
    let mut bundled = String::from("pub const BUNDLED_SKILLS: &[(&str, &[u8])] = &[\n");

    for entry in walkdir::WalkDir::new(&skills_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let rel = entry
            .path()
            .strip_prefix(&skills_root)
            .expect("skill path outside root");
        let rel_str = rel
            .to_string_lossy()
            .replace('\\', "/")
            .replace('"', "\\\"");
        let bytes = fs::read(entry.path()).expect("read skill file failed");
        let encoded = bytes
            .iter()
            .map(|b| b.to_string())
            .collect::<Vec<_>>()
            .join(",");
        bundled.push_str(&format!("    (\"{rel_str}\", &[{encoded}]),\n"));
    }
    bundled.push_str("];\n");
    fs::write(out_dir.join("generated_skills.rs"), bundled).expect("write generated skills failed");
}
