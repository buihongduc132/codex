use clap::CommandFactory;
use clap::Parser;
use clap_complete::Shell;
use clap_complete::generate;
use codex_arg0::arg0_dispatch_or_else;
use codex_chatgpt::apply_command::ApplyCommand;
use codex_chatgpt::apply_command::run_apply_command;
use codex_cli::LandlockCommand;
use codex_cli::SeatbeltCommand;
use codex_cli::login::run_login_status;
use codex_cli::login::run_login_with_api_key;
use codex_cli::login::run_login_with_chatgpt;
use codex_cli::login::run_logout;
use codex_cli::proto;
use codex_common::CliConfigOverrides;
use codex_exec::Cli as ExecCli;
use codex_tui::Cli as TuiCli;
use std::path::PathBuf;

use crate::proto::ProtoCli;

/// Codex CLI
///
/// If no subcommand is specified, options will be forwarded to the interactive CLI.
#[derive(Debug, Parser)]
#[clap(
    author,
    version,
    // If a sub‑command is given, ignore requirements of the default args.
    subcommand_negates_reqs = true,
    // The executable is sometimes invoked via a platform‑specific name like
    // `codex-x86_64-unknown-linux-musl`, but the help output should always use
    // the generic `codex` command name that users run.
    bin_name = "codex"
)]
struct MultitoolCli {
    #[clap(flatten)]
    pub config_overrides: CliConfigOverrides,

    #[clap(flatten)]
    interactive: TuiCli,

    #[clap(subcommand)]
    subcommand: Option<Subcommand>,
}

#[derive(Debug, clap::Subcommand)]
enum Subcommand {
    /// Run Codex non-interactively.
    #[clap(visible_alias = "e")]
    Exec(ExecCli),

    /// Manage login.
    Login(LoginCommand),

    /// Remove stored authentication credentials.
    Logout(LogoutCommand),

    /// Experimental: run Codex as an MCP server.
    Mcp,

    /// Run the Protocol stream via stdin/stdout
    #[clap(visible_alias = "p")]
    Proto(ProtoCli),

    /// Generate shell completion scripts.
    Completion(CompletionCommand),

    /// Internal debugging commands.
    Debug(DebugArgs),

    /// Apply the latest diff produced by Codex agent as a `git apply` to your local working tree.
    #[clap(visible_alias = "a")]
    Apply(ApplyCommand),

    /// Save a conversation under a friendly name for later loading.
    Save(SaveCommand),

    /// Load a saved conversation by name or session id.
    Load(LoadCommand),

    /// Internal: generate TypeScript protocol bindings.
    #[clap(hide = true)]
    GenerateTs(GenerateTsCommand),
}

#[derive(Debug, Parser)]
struct CompletionCommand {
    /// Shell to generate completions for
    #[clap(value_enum, default_value_t = Shell::Bash)]
    shell: Shell,
}

#[derive(Debug, Parser)]
struct DebugArgs {
    #[command(subcommand)]
    cmd: DebugCommand,
}

#[derive(Debug, clap::Subcommand)]
enum DebugCommand {
    /// Run a command under Seatbelt (macOS only).
    Seatbelt(SeatbeltCommand),

    /// Run a command under Landlock+seccomp (Linux only).
    Landlock(LandlockCommand),
}

#[derive(Debug, Parser)]
struct LoginCommand {
    #[clap(skip)]
    config_overrides: CliConfigOverrides,

    #[arg(long = "api-key", value_name = "API_KEY")]
    api_key: Option<String>,

    #[command(subcommand)]
    action: Option<LoginSubcommand>,
}

#[derive(Debug, clap::Subcommand)]
enum LoginSubcommand {
    /// Show login status.
    Status,
}

#[derive(Debug, Parser)]
struct LogoutCommand {
    #[clap(skip)]
    config_overrides: CliConfigOverrides,
}

#[derive(Debug, Parser)]
struct GenerateTsCommand {
    /// Output directory where .ts files will be written
    #[arg(short = 'o', long = "out", value_name = "DIR")]
    out_dir: PathBuf,

    /// Optional path to the Prettier executable to format generated files
    #[arg(short = 'p', long = "prettier", value_name = "PRETTIER_BIN")]
    prettier: Option<PathBuf>,
}

#[derive(Debug, Parser)]
struct SaveCommand {
    /// Name to save the conversation as.
    #[arg(value_name = "NAME")]
    name: String,

    /// Session id to save. If omitted, the latest conversation is used.
    #[arg(long = "id", value_name = "UUID")]
    id: Option<String>,
}

#[derive(Debug, Parser)]
struct LoadCommand {
    /// Name or session id to load.
    #[arg(value_name = "NAME_OR_ID")]
    key: String,
}

fn main() -> anyhow::Result<()> {
    arg0_dispatch_or_else(|codex_linux_sandbox_exe| async move {
        cli_main(codex_linux_sandbox_exe).await?;
        Ok(())
    })
}

async fn cli_main(codex_linux_sandbox_exe: Option<PathBuf>) -> anyhow::Result<()> {
    let cli = MultitoolCli::parse();

    match cli.subcommand {
        None => {
            let mut tui_cli = cli.interactive;
            prepend_config_flags(&mut tui_cli.config_overrides, cli.config_overrides);
            let usage = codex_tui::run_main(tui_cli, codex_linux_sandbox_exe).await?;
            if !usage.is_zero() {
                println!("{}", codex_core::protocol::FinalOutput::from(usage));
            }
        }
        Some(Subcommand::Exec(mut exec_cli)) => {
            prepend_config_flags(&mut exec_cli.config_overrides, cli.config_overrides);
            codex_exec::run_main(exec_cli, codex_linux_sandbox_exe).await?;
        }
        Some(Subcommand::Mcp) => {
            codex_mcp_server::run_main(codex_linux_sandbox_exe, cli.config_overrides).await?;
        }
        Some(Subcommand::Login(mut login_cli)) => {
            prepend_config_flags(&mut login_cli.config_overrides, cli.config_overrides);
            match login_cli.action {
                Some(LoginSubcommand::Status) => {
                    run_login_status(login_cli.config_overrides).await;
                }
                None => {
                    if let Some(api_key) = login_cli.api_key {
                        run_login_with_api_key(login_cli.config_overrides, api_key).await;
                    } else {
                        run_login_with_chatgpt(login_cli.config_overrides).await;
                    }
                }
            }
        }
        Some(Subcommand::Logout(mut logout_cli)) => {
            prepend_config_flags(&mut logout_cli.config_overrides, cli.config_overrides);
            run_logout(logout_cli.config_overrides).await;
        }
        Some(Subcommand::Proto(mut proto_cli)) => {
            prepend_config_flags(&mut proto_cli.config_overrides, cli.config_overrides);
            proto::run_main(proto_cli).await?;
        }
        Some(Subcommand::Completion(completion_cli)) => {
            print_completion(completion_cli);
        }
        Some(Subcommand::Debug(debug_args)) => match debug_args.cmd {
            DebugCommand::Seatbelt(mut seatbelt_cli) => {
                prepend_config_flags(&mut seatbelt_cli.config_overrides, cli.config_overrides);
                codex_cli::debug_sandbox::run_command_under_seatbelt(
                    seatbelt_cli,
                    codex_linux_sandbox_exe,
                )
                .await?;
            }
            DebugCommand::Landlock(mut landlock_cli) => {
                prepend_config_flags(&mut landlock_cli.config_overrides, cli.config_overrides);
                codex_cli::debug_sandbox::run_command_under_landlock(
                    landlock_cli,
                    codex_linux_sandbox_exe,
                )
                .await?;
            }
        },
        Some(Subcommand::Apply(mut apply_cli)) => {
            prepend_config_flags(&mut apply_cli.config_overrides, cli.config_overrides);
            run_apply_command(apply_cli, None).await?;
        }
        Some(Subcommand::Save(cmd)) => {
            handle_save(cmd).await?;
        }
        Some(Subcommand::Load(cmd)) => {
            handle_load(cmd).await?;
        }
        Some(Subcommand::GenerateTs(gen_cli)) => {
            codex_protocol_ts::generate_ts(&gen_cli.out_dir, gen_cli.prettier.as_deref())?;
        }
    }

    Ok(())
}

/// Prepend root-level overrides so they have lower precedence than
/// CLI-specific ones specified after the subcommand (if any).
fn prepend_config_flags(
    subcommand_config_overrides: &mut CliConfigOverrides,
    cli_config_overrides: CliConfigOverrides,
) {
    subcommand_config_overrides
        .raw_overrides
        .splice(0..0, cli_config_overrides.raw_overrides);
}

fn print_completion(cmd: CompletionCommand) {
    let mut app = MultitoolCli::command();
    let name = "codex";
    generate(cmd.shell, &mut app, name, &mut std::io::stdout());
}

async fn handle_save(cmd: SaveCommand) -> anyhow::Result<()> {
    use codex_core::RolloutRecorder;
    use codex_core::config::find_codex_home;

    let codex_home = find_codex_home()?;
    // Resolve target path by id or by selecting latest
    let target_path = if let Some(id_prefix) = cmd.id.as_deref() {
        // Scan a large page and match by session id prefix from the meta head.
        let page = RolloutRecorder::list_conversations(&codex_home, 10_000, None).await?;
        let mut found: Option<std::path::PathBuf> = None;
        for it in &page.items {
            if let Some(first) = it.head.first() {
                let sid = first
                    .get("meta")
                    .and_then(|m| m.get("id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if sid.starts_with(id_prefix) || sid == id_prefix {
                    found = Some(it.path.clone());
                    break;
                }
            }
        }
        found.ok_or_else(|| anyhow::anyhow!("session id not found"))?
    } else {
        let page = RolloutRecorder::list_conversations(&codex_home, 1, None).await?;
        page.items
            .first()
            .map(|it| it.path.clone())
            .ok_or_else(|| anyhow::anyhow!("no conversations found"))?
    };

    // Write/update saves index
    let saves_path = codex_home.join("saves.json");
    let mut root: serde_json::Value = if saves_path.exists() {
        serde_json::from_str(&std::fs::read_to_string(&saves_path)?)
            .unwrap_or_else(|_| serde_json::json!({"by_name":{}, "by_id":{}}))
    } else {
        serde_json::json!({"by_name":{}, "by_id":{}})
    };
    // Insert name and id mapping
    let by_name = root["by_name"].as_object_mut().unwrap();
    by_name.insert(
        cmd.name.clone(),
        serde_json::Value::String(target_path.to_string_lossy().to_string()),
    );
    // Best-effort id: infer from filename
    if let Some(file) = target_path.file_name().and_then(|s| s.to_str()) {
        if let Some(id) = file
            .split('-')
            .last()
            .and_then(|s| s.strip_suffix(".jsonl"))
        {
            root["by_id"][id] =
                serde_json::Value::String(target_path.to_string_lossy().to_string());
        }
    }
    std::fs::write(saves_path, serde_json::to_string_pretty(&root)?)?;
    println!("saved `{}` -> {}", cmd.name, target_path.display());
    Ok(())
}

async fn handle_load(cmd: LoadCommand) -> anyhow::Result<()> {
    use codex_core::config::find_codex_home;

    let codex_home = find_codex_home()?;
    let saves_path = codex_home.join("saves.json");
    let path = if saves_path.exists() {
        let root: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&saves_path)?)?;
        let by_name = root
            .get("by_name")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();
        let by_id = root
            .get("by_id")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();
        by_name
            .get(&cmd.key)
            .or_else(|| by_id.get(&cmd.key))
            .and_then(|v| v.as_str())
            .map(std::path::PathBuf::from)
            .ok_or_else(|| anyhow::anyhow!("save not found: {}", cmd.key))?
    } else {
        anyhow::bail!("no saves.json present; use `codex save <name>` first");
    };

    // Build a TUI CLI that loads the specified path.
    let mut tui_cli = TuiCli::parse_from(["codex", "--load-path", &path.to_string_lossy()]);
    // Forward any root-level overrides later if needed.
    let usage = codex_tui::run_main(tui_cli, None).await?;
    if !usage.is_zero() {
        println!("{}", codex_core::protocol::FinalOutput::from(usage));
    }
    Ok(())
}
