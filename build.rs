use std::process::Command;

/// Embed build metadata so the binary can report exactly which build it is.
/// Cargo's package version alone can't distinguish `v0.2.0-build.N` releases,
/// which all share the same version — so we also capture the git commit and
/// (in CI) the release build number.
fn main() {
    let git_hash = Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=GIT_HASH={git_hash}");

    // Rebuild the version string when the checked-out commit changes.
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs");
    // GITHUB_RUN_NUMBER is read via option_env! in the crate; re-run if it changes.
    println!("cargo:rerun-if-env-changed=GITHUB_RUN_NUMBER");
}
