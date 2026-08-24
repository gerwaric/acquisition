//! Stamp the git commit into the binary so a journal, a log line, or a
//! `--version` can say which code produced it. Checking the checkout is not
//! enough: the rung-8 soak ran a stale binary on a fresh checkout.
use std::process::Command;

fn main() {
    let hash = Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let dirty = Command::new("git")
        .args(["status", "--porcelain", "--untracked-files=no"])
        .output()
        .ok()
        .is_some_and(|o| o.status.success() && !o.stdout.is_empty());
    let build = if dirty { format!("{hash}-dirty") } else { hash };
    println!("cargo:rustc-env=ACQ_BUILD={build}");
    // Re-stamp on commit, checkout, or staging. A worktree's `.git` may be a
    // file, so ask git where the real directory is rather than guessing.
    if let Some(git_dir) = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    {
        for name in ["HEAD", "index"] {
            println!("cargo:rerun-if-changed={git_dir}/{name}");
        }
    }
}
