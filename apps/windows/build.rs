use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let repo = manifest_dir.join("../..");

    println!("cargo:rerun-if-changed=build.rs");
    emit_source_rerun_triggers(&repo);
    emit_git_rerun_triggers(&repo);

    let head = git_stdout(&repo, &["rev-parse", "HEAD"]);
    let status = git_stdout(&repo, &["status", "--porcelain", "--untracked-files=all"]);
    let state = match (head.as_deref(), status.as_deref()) {
        (Some(_), Some("")) => "clean",
        (Some(_), Some(_)) => "dirty",
        _ => "unavailable",
    };
    let sha = head.unwrap_or_else(|| "unavailable".to_string());

    println!("cargo:rustc-env=POWDERGAME_BUILD_SOURCE_SHA={sha}");
    println!("cargo:rustc-env=POWDERGAME_BUILD_GIT_STATE={state}");
}

/// Watch every source path Git knew about at this build, including existing
/// untracked non-ignored inputs. HEAD/index/ref triggers below cover checkout
/// and staging changes. A previously built EXE intentionally keeps these
/// values: it reports its own build source, never the checkout at run time.
fn emit_source_rerun_triggers(repo: &Path) {
    let Some(output) = git_output(
        repo,
        &[
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ],
    ) else {
        println!("cargo:rerun-if-changed=src");
        println!("cargo:rerun-if-changed=Cargo.toml");
        return;
    };
    for raw_path in output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|p| !p.is_empty())
    {
        let path = String::from_utf8_lossy(raw_path);
        println!(
            "cargo:rerun-if-changed={}",
            repo.join(path.as_ref()).display()
        );
    }
}

fn emit_git_rerun_triggers(repo: &Path) {
    for git_path in ["HEAD", "index", "packed-refs"] {
        if let Some(path) = git_stdout(repo, &["rev-parse", "--git-path", git_path]) {
            println!(
                "cargo:rerun-if-changed={}",
                resolve_git_path(repo, &path).display()
            );
        }
    }
    if let Some(reference) = git_stdout(repo, &["symbolic-ref", "-q", "HEAD"]) {
        if let Some(path) = git_stdout(repo, &["rev-parse", "--git-path", &reference]) {
            println!(
                "cargo:rerun-if-changed={}",
                resolve_git_path(repo, &path).display()
            );
        }
    }
}

fn resolve_git_path(repo: &Path, value: &str) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        repo.join(path)
    }
}

fn git_stdout(repo: &Path, args: &[&str]) -> Option<String> {
    let output = git_output(repo, args)?;
    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_string())
}

fn git_output(repo: &Path, args: &[&str]) -> Option<std::process::Output> {
    let safe_directory = repo
        .canonicalize()
        .unwrap_or_else(|_| repo.to_path_buf())
        .to_string_lossy()
        .replace('\\', "/");
    let output = Command::new("git")
        .arg("-c")
        .arg(format!("safe.directory={safe_directory}"))
        .args(args)
        .current_dir(repo)
        .output()
        .ok()?;
    output.status.success().then_some(output)
}
