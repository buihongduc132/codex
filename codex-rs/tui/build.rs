use std::process::Command;

fn main() {
    // Attempt to resolve the current git commit (short) and commit timestamp (ISO 8601).
    // If these commands fail (e.g., not a git checkout), fall back to placeholder values.
    let commit = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    let datetime = Command::new("git")
        .args(["show", "-s", "--format=%cI", "HEAD"]) // committer date in strict ISO 8601
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=CODEX_BUILD_COMMIT={}", commit);
    println!("cargo:rustc-env=CODEX_BUILD_DATETIME={}", datetime);
}
