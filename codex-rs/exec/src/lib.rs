mod cli;
mod event_processor;
mod event_processor_with_human_output;
mod event_processor_with_json_output;
mod exec_config;

use std::io::IsTerminal;
use std::io::Read;
use std::path::PathBuf;

pub use cli::Cli;
use codex_core::BUILT_IN_OSS_MODEL_PROVIDER_ID;
use codex_core::ConversationManager;
use codex_core::NewConversation;
use codex_core::config::Config;
use codex_core::config::ConfigOverrides;
use codex_core::git_info::get_git_repo_root;
use codex_core::protocol::AskForApproval;
use codex_core::protocol::Event;
use codex_core::protocol::EventMsg;
use codex_core::protocol::InputItem;
use codex_core::protocol::Op;
use codex_core::protocol::TaskCompleteEvent;
use codex_ollama::DEFAULT_OSS_MODEL;
use codex_protocol::config_types::SandboxMode;
use event_processor_with_human_output::EventProcessorWithHumanOutput;
use event_processor_with_json_output::EventProcessorWithJsonOutput;
use tracing::debug;
use tracing::error;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::event_processor::CodexStatus;
use crate::event_processor::EventProcessor;

pub async fn run_main(cli: Cli, codex_linux_sandbox_exe: Option<PathBuf>) -> anyhow::Result<()> {
    let Cli {
        images,
        model: model_cli_arg,
        oss,
        config_profile,
        full_auto,
        dangerously_bypass_approvals_and_sandbox,
        cwd,
        skip_git_repo_check,
        color,
        last_message_file,
        json: json_mode,
        sandbox_mode: sandbox_mode_cli_arg,
        prompt,
        config_overrides,
        auto_summary,
        summarize_name,
        from_summarize,
    } = cli;

    // Determine the prompt based on CLI arg and/or stdin.
    let prompt = match prompt {
        Some(p) if p != "-" => p,
        // Either `-` was passed or no positional arg.
        maybe_dash => {
            // When no arg (None) **and** stdin is a TTY, bail out early – unless the
            // user explicitly forced reading via `-`.
            let force_stdin = matches!(maybe_dash.as_deref(), Some("-"));

            if std::io::stdin().is_terminal() && !force_stdin {
                eprintln!(
                    "No prompt provided. Either specify one as an argument or pipe the prompt into stdin."
                );
                std::process::exit(1);
            }

            // Ensure the user knows we are waiting on stdin, as they may
            // have gotten into this state by mistake. If so, and they are not
            // writing to stdin, Codex will hang indefinitely, so this should
            // help them debug in that case.
            if !force_stdin {
                eprintln!("Reading prompt from stdin...");
            }
            let mut buffer = String::new();
            if let Err(e) = std::io::stdin().read_to_string(&mut buffer) {
                eprintln!("Failed to read prompt from stdin: {e}");
                std::process::exit(1);
            } else if buffer.trim().is_empty() {
                eprintln!("No prompt provided via stdin.");
                std::process::exit(1);
            }
            buffer
        }
    };

    let (stdout_with_ansi, stderr_with_ansi) = match color {
        cli::Color::Always => (true, true),
        cli::Color::Never => (false, false),
        cli::Color::Auto => (
            std::io::stdout().is_terminal(),
            std::io::stderr().is_terminal(),
        ),
    };

    // TODO(mbolin): Take a more thoughtful approach to logging.
    let default_level = "error";
    let _ = tracing_subscriber::fmt()
        // Fallback to the `default_level` log filter if the environment
        // variable is not set _or_ contains an invalid value
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .or_else(|_| EnvFilter::try_new(default_level))
                .unwrap_or_else(|_| EnvFilter::new(default_level)),
        )
        .with_ansi(stderr_with_ansi)
        .with_writer(std::io::stderr)
        .try_init();

    let sandbox_mode = if full_auto {
        Some(SandboxMode::WorkspaceWrite)
    } else if dangerously_bypass_approvals_and_sandbox {
        Some(SandboxMode::DangerFullAccess)
    } else {
        sandbox_mode_cli_arg.map(Into::<SandboxMode>::into)
    };

    // When using `--oss`, let the bootstrapper pick the model (defaulting to
    // gpt-oss:20b) and ensure it is present locally. Also, force the built‑in
    // `oss` model provider.
    let model = if let Some(model) = model_cli_arg {
        Some(model)
    } else if oss {
        Some(DEFAULT_OSS_MODEL.to_owned())
    } else {
        None // No model specified, will use the default.
    };

    let model_provider = if oss {
        Some(BUILT_IN_OSS_MODEL_PROVIDER_ID.to_string())
    } else {
        None // No specific model provider override.
    };

    // Load configuration and determine approval policy
    let overrides = ConfigOverrides {
        model,
        config_profile,
        // This CLI is intended to be headless and has no affordances for asking
        // the user for approval.
        approval_policy: Some(AskForApproval::Never),
        sandbox_mode,
        cwd: cwd.map(|p| p.canonicalize().unwrap_or(p)),
        model_provider,
        codex_linux_sandbox_exe,
        base_instructions: None,
        include_plan_tool: None,
        include_apply_patch_tool: None,
        include_view_image_tool: None,
        disable_response_storage: None,
        show_raw_agent_reasoning: oss.then_some(true),
        tools_web_search_request: None,
    };
    // Parse `-c` overrides.
    let cli_kv_overrides = match config_overrides.parse_overrides() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error parsing -c overrides: {e}");
            std::process::exit(1);
        }
    };

    let config = Config::load_with_cli_overrides(cli_kv_overrides, overrides)?;

    // Load exec-specific overrides from config.toml (e.g., ~/.qoo/config.toml under qoo).
    let exec_overrides = exec_config::load_exec_overrides(&config.codex_home);
    let auto_summary = auto_summary.or(exec_overrides.auto_summary);
    let mut event_processor: Box<dyn EventProcessor> = if json_mode {
        Box::new(EventProcessorWithJsonOutput::new(last_message_file.clone()))
    } else {
        Box::new(EventProcessorWithHumanOutput::create_with_ansi(
            stdout_with_ansi,
            &config,
            last_message_file.clone(),
        ))
    };

    if oss {
        // Emit a banner with MOODED build metadata for quick version verification.
        {
            let build_sha = env!("CODEX_BUILD_SHA");
            let build_time = env!("CODEX_BUILD_TIME");
            let branch = env!("CODEX_BUILD_BRANCH");
            let cwd_display = {
                let sep = std::path::MAIN_SEPARATOR;
                if let Some(home) = dirs::home_dir() {
                    if let Ok(rel) = config.cwd.strip_prefix(home) {
                        format!("~{sep}{}", rel.display())
                    } else {
                        config.cwd.display().to_string()
                    }
                } else {
                    config.cwd.display().to_string()
                }
            };
            println!(
                ">_ [MOODED Build: {build_sha} {build_time}] You are using OpenAI Codex in {cwd_display} (branch: {branch})\n"
            );
        }
        codex_ollama::ensure_oss_ready(&config)
            .await
            .map_err(|e| anyhow::anyhow!("OSS setup failed: {e}"))?;
    }

    // Print the effective configuration and prompt so users can see what Codex
    // is using.
    event_processor.print_config_summary(&config, &prompt);

    if !skip_git_repo_check && get_git_repo_root(&config.cwd.to_path_buf()).is_none() {
        eprintln!("Not inside a trusted directory and --skip-git-repo-check was not specified.");
        std::process::exit(1);
    }

    let conversation_manager = ConversationManager::new(codex_core::AuthManager::shared(
        config.codex_home.clone(),
        config.preferred_auth_method,
        config.responses_originator_header.clone(),
    ));
    let NewConversation {
        conversation_id: _,
        conversation,
        session_configured,
    } = conversation_manager.new_conversation(config).await?;
    info!("Codex initialized with event: {session_configured:?}");

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Event>();
    {
        let conversation = conversation.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {
                        tracing::debug!("Keyboard interrupt");
                        // Immediately notify Codex to abort any in‑flight task.
                        conversation.submit(Op::Interrupt).await.ok();

                        // Exit the inner loop and return to the main input prompt. The codex
                        // will emit a `TurnInterrupted` (Error) event which is drained later.
                        break;
                    }
                    res = conversation.next_event() => match res {
                        Ok(event) => {
                            debug!("Received event: {event:?}");

                            let is_shutdown_complete = matches!(event.msg, EventMsg::ShutdownComplete);
                            if let Err(e) = tx.send(event) {
                                error!("Error sending event: {e:?}");
                                break;
                            }
                            if is_shutdown_complete {
                                info!("Received shutdown event, exiting event loop.");
                                break;
                            }
                        },
                        Err(e) => {
                            error!("Error receiving event: {e:?}");
                            break;
                        }
                    }
                }
            }
        });
    }

    // Send images first, if any.
    if !images.is_empty() {
        let items: Vec<InputItem> = images
            .into_iter()
            .map(|path| InputItem::LocalImage { path })
            .collect();
        let initial_images_event_id = conversation.submit(Op::UserInput { items }).await?;
        info!("Sent images with event ID: {initial_images_event_id}");
        while let Ok(event) = conversation.next_event().await {
            if event.id == initial_images_event_id
                && matches!(
                    event.msg,
                    EventMsg::TaskComplete(TaskCompleteEvent {
                        last_agent_message: _,
                    })
                )
            {
                break;
            }
        }
    }

    // Send the prompt.
    let items: Vec<InputItem> = vec![InputItem::Text { text: prompt }];
    let initial_prompt_task_id = conversation.submit(Op::UserInput { items }).await?;
    info!("Sent prompt with event ID: {initial_prompt_task_id}");

    // Track execution stats to feed into an optional rich auto-summary.
    #[derive(Default)]
    struct ExecStats {
        exec_calls: usize,
        mcp_calls: usize,
        patches_applied: usize,
        errors: usize,
    }
    let mut stats = ExecStats::default();
    let mut ran_auto_summary = false;

    // Run the loop until the task is complete (and optional auto-summary is done).
    while let Some(event) = rx.recv().await {
        // Lightweight accounting for a richer summary.
        match &event.msg {
            EventMsg::ExecCommandBegin(_) => stats.exec_calls += 1,
            EventMsg::McpToolCallBegin(_) => stats.mcp_calls += 1,
            EventMsg::PatchApplyBegin(_) => stats.patches_applied += 1,
            EventMsg::Error(_) | EventMsg::StreamError(_) => stats.errors += 1,
            _ => {}
        }

        let is_task_complete =
            matches!(event.msg, EventMsg::TaskComplete(TaskCompleteEvent { .. }));
        let shutdown: CodexStatus = event_processor.process_event(event);

        if is_task_complete && auto_summary.is_some() && !ran_auto_summary {
            // Prefer a normal chat turn for auto-summary to avoid upstream `instructions`
            // constraints and to keep behavior consistent with regular prompts.
            let prompt_text = match auto_summary {
                Some(crate::cli::AutoSummary::Rich) => {
                    // Prefer file override if provided.
                    if let Some(path) = exec_overrides.summary_rich_file.as_ref() {
                        match std::fs::read_to_string(path) {
                            Ok(mut s) => {
                                // Append stats to give the model extra context.
                                s.push_str(&format!(
                                    "\nStats: commands={commands}, tools={tools}, patches={patches}, errors={errors}",
                                    commands = stats.exec_calls,
                                    tools = stats.mcp_calls,
                                    patches = stats.patches_applied,
                                    errors = stats.errors
                                ));
                                s
                            }
                            Err(_) => default_rich_summary_prompt(
                                stats.exec_calls,
                                stats.mcp_calls,
                                stats.patches_applied,
                                stats.errors,
                            ),
                        }
                    } else {
                        default_rich_summary_prompt(
                            stats.exec_calls,
                            stats.mcp_calls,
                            stats.patches_applied,
                            stats.errors,
                        )
                    }
                }
                Some(crate::cli::AutoSummary::Brief) | None => {
                    if let Some(path) = exec_overrides.summary_brief_file.as_ref() {
                        std::fs::read_to_string(path)
                            .unwrap_or_else(|_| default_brief_summary_prompt())
                    } else {
                        default_brief_summary_prompt()
                    }
                }
            };

            let items: Vec<InputItem> = vec![InputItem::Text { text: prompt_text }];
            let _ = conversation.submit(Op::UserInput { items }).await?;
            ran_auto_summary = true;
            continue;
        }

        match shutdown {
            CodexStatus::Running => continue,
            CodexStatus::InitiateShutdown => {
                conversation.submit(Op::Shutdown).await?;
            }
            CodexStatus::Shutdown => {
                break;
            }
        }
    }

    Ok(())
}

fn default_rich_summary_prompt(
    commands: usize,
    tools: usize,
    patches: usize,
    errors: usize,
) -> String {
    format!(
        concat!(
            "Summarize the conversation so far for the human.",
            " Start with a one-line outcome, then 3-6 concise bullets.",
            " Include a short stats line with counts for commands, tools, patches, and errors.",
            " Keep it actionable and avoid repeating raw logs.\n",
            "Stats: commands={commands}, tools={tools}, patches={patches}, errors={errors}"
        ),
        commands = commands,
        tools = tools,
        patches = patches,
        errors = errors
    )
}

fn default_brief_summary_prompt() -> String {
    "Please summarize the conversation so far in 3-5 concise bullets with next steps.".to_string()
}
