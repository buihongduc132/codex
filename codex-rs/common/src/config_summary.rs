use std::path::{Path, PathBuf};

use codex_core::WireApi;
use codex_core::config::Config;
use codex_core::git_info::get_git_repo_root;

use crate::sandbox_summary::summarize_sandbox_policy;

/// Build a list of key/value pairs summarizing the effective configuration.
pub fn create_config_summary_entries(config: &Config) -> Vec<(&'static str, String)> {
    let mut entries = vec![
        ("workdir", config.cwd.display().to_string()),
        ("model", config.model.clone()),
        ("provider", config.model_provider_id.clone()),
        ("approval", config.approval_policy.to_string()),
        ("sandbox", summarize_sandbox_policy(&config.sandbox_policy)),
    ];

    // Add Git context if present (repo root and branch/detached HEAD).
    if let (Some(repo_root), branch) = detect_git_context(&config.cwd) {
        entries.push(("git repo", repo_root.display().to_string()));
        if let Some(b) = branch {
            entries.push(("git branch", b));
        }
    }
    if config.model_provider.wire_api == WireApi::Responses
        && config.model_family.supports_reasoning_summaries
    {
        entries.push((
            "reasoning effort",
            config.model_reasoning_effort.to_string(),
        ));
        entries.push((
            "reasoning summaries",
            config.model_reasoning_summary.to_string(),
        ));
    }

    entries
}

/// Detect git repository root and branch (or detached HEAD SHA) for the given cwd.
/// This function does not spawn external commands; it inspects `.git/HEAD` directly.
pub fn detect_git_context(cwd: &Path) -> (Option<PathBuf>, Option<String>) {
    let Some(repo_root) = get_git_repo_root(cwd) else {
        return (None, None);
    };

    let head_path = repo_root.join(".git").join("HEAD");
    match std::fs::read_to_string(&head_path) {
        Ok(s) => {
            let line = s.trim();
            if let Some(rest) = line.strip_prefix("ref: ") {
                // Expect a path like "refs/heads/main"; take the last segment as branch name.
                let branch = Path::new(rest)
                    .file_name()
                    .and_then(|os| os.to_str())
                    .map(|s| s.to_string());
                (Some(repo_root), branch)
            } else if !line.is_empty() {
                // Detached HEAD: return the raw content (SHA or ref) truncated for readability.
                let short = if line.len() > 12 { &line[..12] } else { line };
                (Some(repo_root), Some(format!("detached:{short}")))
            } else {
                (Some(repo_root), None)
            }
        }
        Err(_) => (Some(repo_root), None),
    }
}

#[cfg(test)]
mod tests {
    use super::detect_git_context;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn detect_git_branch_from_head_ref() {
        let dir = tempdir().unwrap();
        let git_dir = dir.path().join(".git");
        fs::create_dir_all(&git_dir).unwrap();
        fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n").unwrap();

        let (repo, branch) = detect_git_context(dir.path());
        assert_eq!(repo.as_deref(), Some(dir.path()));
        assert_eq!(branch.as_deref(), Some("main"));
    }

    #[test]
    fn detect_git_detached_head() {
        let dir = tempdir().unwrap();
        let git_dir = dir.path().join(".git");
        fs::create_dir_all(&git_dir).unwrap();
        fs::write(
            git_dir.join("HEAD"),
            "2f1c3b4a5d6e7f8091a2b3c4d5e6f7089abcdeff\n",
        )
        .unwrap();

        let (repo, branch) = detect_git_context(dir.path());
        assert_eq!(repo.as_deref(), Some(dir.path()));
        let b = branch.expect("expected detached head");
        assert!(b.starts_with("detached:"));
    }
}
