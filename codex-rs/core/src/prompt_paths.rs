//! Exposes absolute file paths for built-in prompts (resolved at build time).

/// Absolute path to the built-in system instructions file (prompt.md).
pub const SYSTEM_INSTRUCTIONS_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/prompt.md");

/// Absolute path to the built-in compact command prompt file.
pub const COMPACT_PROMPT_PATH: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/src/prompt_for_compact_command.md");

