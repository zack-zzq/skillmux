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
}
