use clap::Parser;
use codex_arg0::arg0_dispatch_or_else;
use codex_common::CliConfigOverrides;
use codex_tui::Cli;
use codex_tui::run_main;

#[derive(Parser, Debug)]
struct TopCli {
    #[clap(flatten)]
    config_overrides: CliConfigOverrides,

    #[clap(flatten)]
    inner: Cli,
}

fn main() -> anyhow::Result<()> {
    arg0_dispatch_or_else(|codex_linux_sandbox_exe| async move {
        let top_cli = TopCli::parse();
        let mut inner = top_cli.inner;
        inner
            .config_overrides
            .raw_overrides
            .splice(0..0, top_cli.config_overrides.raw_overrides);
        // Non‑interactive status mode: support either --status flag or
        // a positional prompt of "status"/"--status".
        if inner.status
            || matches!(inner.prompt.as_deref(), Some("status") | Some("--status"))
        {
            let s = codex_tui::status_string(&inner, codex_linux_sandbox_exe)?;
            println!("{s}");
            return Ok(());
        }
        let usage = run_main(inner, codex_linux_sandbox_exe).await?;
        if !usage.is_zero() {
            println!("{}", codex_core::protocol::FinalOutput::from(usage));
        }
        Ok(())
    })
}
