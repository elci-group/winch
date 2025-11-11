cat ~/winch/src/main.rs
use std::process::{Command, Stdio};
use std::fs;
use serde_json::Value;
use toml_edit::{Document, value};
use regex::Regex;
use reqwest::Client;
use std::path::PathBuf;
use anyhow::Result;
use semver::Version;
const MAX_ROLLBACKS: usize = 5;

#[tokio::main]
async fn main() -> Result<()> {
    // --- CLI Args ---
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return Ok(());
    }

    let dir = if let Some(pos) = args.iter().position(|a| a == "--dir") {
        args.get(pos + 1).map(|s| PathBuf::from(s)).unwrap_or(std::env::current_dir()?)
    } else {
        std::env::current_dir()?
    };

    println!("🛠️  Running Winch in directory: {}", dir.display());

    // --- Step 1: Initial build attempt ---
    let output = Command::new("cargo")
        .arg("build")
        .current_dir(&dir)
        .stderr(Stdio::piped())
        .output()?;

    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        println!("✅ Cargo build succeeded! No dependency issues detected.");
        return Ok(());
    }

    println!("⚠️  Build failed. Detecting dependency issues...");

    // --- Step 2: Detect problems ---
    let conflict_crates = parse_conflicts(&stderr);
    let missing_crates = parse_missing_crates(&stderr);

    let mut problem_crates = conflict_crates.clone();
    problem_crates.extend(missing_crates.iter().cloned());

    if problem_crates.is_empty() {
        println!("❌ No parseable dependency issues found.");
        return Ok(());
    }

    println!("🧩 Problematic crates detected: {:?}", problem_crates);

    // --- Step 3: Fetch candidate versions ---
    let client = Client::new();
    let mut candidates_map = std::collections::HashMap::new();
    for crate_name in &problem_crates {
        let mut versions = get_candidate_versions(&client, crate_name).await?;

        // For missing crates, ensure latest is first
        if missing_crates.contains(crate_name) {
            versions.sort_by(|a, b| Version::parse(b).unwrap().cmp(&Version::parse(a).unwrap()));

        }

        candidates_map.insert(crate_name.clone(), versions);
    }

    // --- Step 4: Generate version combinations ---
    let all_combinations = generate_combinations(&candidates_map);
    println!("🔄 Trying {} version combinations...", all_combinations.len());

    // --- Step 5: Try combinations ---
    for combo in all_combinations {
        println!("🧪 Trying combination: {:?}", combo);

        let cargo_toml_path = dir.join("Cargo.toml");
        let cargo_toml_content = fs::read_to_string(&cargo_toml_path)?;
        let mut doc = cargo_toml_content.parse::<Document>()?;

        for (crate_name, version) in &combo {
            doc["dependencies"][crate_name] = value(version.clone());
        }

        let winch_toml_path = dir.join("Cargo.winch.toml");
        fs::write(&winch_toml_path, doc.to_string())?;

        let build_result = Command::new("cargo")
            .arg("build")
            .arg("--manifest-path")
            .arg(&winch_toml_path)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .current_dir(&dir)
            .spawn()?
            .wait()?;

        if build_result.success() {
            println!("🎉 Build succeeded with combination: {:?}", combo);
            fs::write(&cargo_toml_path, doc.to_string())?;
            println!("📦 Cargo.toml updated with working versions.");
            return Ok(());
        } else {
            println!("❌ Build failed. Trying next combination...");
        }
    }

    println!("💀 All combinations failed. Manual intervention required.");
    Ok(())
}

/// Print help message
fn print_help() {
    println!("🛠️  Winch — Automatic Cargo Dependency Resolver");
    println!();
    println!("Usage:");
    println!("  winch [--dir <path>] [--help|-h]");
    println!();
    println!("Options:");
    println!("  --dir <path>   Path to Rust project (default: current dir)");
    println!("  --help, -h     Show this help message");
}

/// Parse cargo stderr for conflicting crates
fn parse_conflicts(stderr: &str) -> Vec<String> {
    let re = Regex::new(r#"failed to select a version for `([^`]*)`"#).unwrap();
    re.captures_iter(stderr)
        .map(|cap| cap[1].to_string())
        .collect()
}

/// Parse cargo stderr for missing crates
fn parse_missing_crates(stderr: &str) -> Vec<String> {
    let mut missing = vec![];
    let re1 = Regex::new(r#"can't find crate for `([^`]*)`"#).unwrap();
    missing.extend(re1.captures_iter(stderr).map(|cap| cap[1].to_string()));
    let re2 = Regex::new(r#"could not find `([^`]*)` in registry"#).unwrap();
    missing.extend(re2.captures_iter(stderr).map(|cap| cap[1].to_string()));
    missing
}

/// Fetch top candidate versions from crates.io
async fn get_candidate_versions(client: &Client, crate_name: &str) -> Result<Vec<String>> {
    let url = format!("https://crates.io/api/v1/crates/{}", crate_name);
    let resp: Value = client.get(&url).send().await?.json().await?;

    let versions = resp["versions"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| {
            let vers = v["num"].as_str()?;
            let yanked = v["yanked"].as_bool().unwrap_or(false);
            if !yanked { Some(vers.to_string()) } else { None }
        })
        .take(MAX_ROLLBACKS)
        .collect::<Vec<String>>();

    Ok(versions)
}

/// Generate all combinations of candidate versions
fn generate_combinations(
    candidates_map: &std::collections::HashMap<String, Vec<String>>
) -> Vec<std::collections::HashMap<String, String>> {
    let keys: Vec<&String> = candidates_map.keys().collect();
    let mut lists: Vec<&Vec<String>> = vec![];
    for k in &keys { lists.push(candidates_map.get(*k).unwrap()); }

    let mut combos = vec![];
    let mut indices = vec![0; lists.len()];

    loop {
        let mut combo = std::collections::HashMap::new();
        for (i, &key) in keys.iter().enumerate() {
            combo.insert(key.clone(), lists[i][indices[i]].clone());
        }
        combos.push(combo);

        let mut carry = 1;
        for i in 0..indices.len() {
            if carry == 0 { break; }
            indices[i] += carry;
            if indices[i] >= lists[i].len() {
                indices[i] = 0;
            } else {
                carry = 0;
            }
        }
        if carry == 1 { break; }
    }

    combos
}
