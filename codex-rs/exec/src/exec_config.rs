use serde::Deserialize;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize)]
struct ExecPromptsCfg {
    pub summary_rich_file: Option<PathBuf>,
    pub summary_brief_file: Option<PathBuf>,
}

#[derive(Debug, Default, Deserialize)]
struct ExecSectionCfg {
    pub auto_summary: Option<String>,
    pub prompts: Option<ExecPromptsCfg>,
}

#[derive(Debug, Default, Deserialize)]
struct RootCfg {
    pub exec: Option<ExecSectionCfg>,
}

#[derive(Debug, Default, Clone)]
pub struct ExecOverrides {
    pub auto_summary: Option<crate::cli::AutoSummary>,
    pub summary_rich_file: Option<PathBuf>,
    pub summary_brief_file: Option<PathBuf>,
}

pub fn load_exec_overrides(codex_home: &Path) -> ExecOverrides {
    let cfg_path = codex_home.join("config.toml");
    let mut out = ExecOverrides::default();
    if !cfg_path.exists() {
        return out;
    }
    let text = match std::fs::read_to_string(&cfg_path) {
        Ok(s) => s,
        Err(_) => return out,
    };
    let parsed: RootCfg = match toml::from_str(&text) {
        Ok(v) => v,
        Err(_) => return out,
    };
    if let Some(exec) = parsed.exec {
        // auto_summary
        out.auto_summary = exec.auto_summary.as_deref().and_then(|s| match s {
            "rich" => Some(crate::cli::AutoSummary::Rich),
            "brief" => Some(crate::cli::AutoSummary::Brief),
            _ => None,
        });
        if let Some(prompts) = exec.prompts {
            out.summary_rich_file = prompts.summary_rich_file;
            out.summary_brief_file = prompts.summary_brief_file;
        }
    }
    out
}
