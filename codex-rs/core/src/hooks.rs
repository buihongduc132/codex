use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HooksConfig {
    pub session_start: Option<HookSpec>,
    pub session_end: Option<HookSpec>,
    pub pre_command: Option<HookSpec>,
    pub post_command: Option<HookSpec>,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct HooksToml {
    pub session_start: Option<HookToml>,
    pub session_end: Option<HookToml>,
    pub pre_command: Option<HookToml>,
    pub post_command: Option<HookToml>,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct HookToml {
    pub command: Option<Vec<String>>,
    pub route: Option<String>,
    pub timeout_ms: Option<u64>,
    pub on_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookSpec {
    pub command: Vec<String>,
    pub route: HookRoute,
    pub timeout_ms: u64,
    pub on_error: HookOnError,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookRoute {
    Ui,
    Llm,
    Both,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookOnError {
    FailOpen,
    FailClosed,
}
