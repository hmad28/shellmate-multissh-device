use serde::Serialize;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitInfo {
    pub branch: Option<String>,
    pub has_changes: bool,
    pub ahead: u32,
    pub behind: u32,
}

#[tauri::command]
pub async fn git_get_info(path: Option<String>) -> GitInfo {
    let dir = path.as_deref().unwrap_or(".");
    let branch = run_git(&["rev-parse", "--abbrev-ref", "HEAD"], dir).await;

    if branch.is_none() {
        return GitInfo {
            branch: None,
            has_changes: false,
            ahead: 0,
            behind: 0,
        };
    }

    let has_changes = run_git(&["status", "--porcelain"], dir)
        .await
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);

    let (ahead, behind) = match run_git(
        &["rev-list", "--left-right", "--count", "HEAD...@{upstream}"],
        dir,
    )
    .await
    {
        Some(output) => {
            let parts: Vec<&str> = output.trim().split('\t').collect();
            (
                parts.first().and_then(|s| s.parse().ok()).unwrap_or(0),
                parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0),
            )
        }
        None => (0, 0),
    };

    GitInfo {
        branch,
        has_changes,
        ahead,
        behind,
    }
}

async fn run_git(args: &[&str], dir: &str) -> Option<String> {
    let mut cmd = Command::new("git");
    cmd.args(args).current_dir(dir);
    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    let output = cmd.output().await.ok()?;
    if output.status.success() {
        String::from_utf8(output.stdout).ok()
    } else {
        None
    }
}
