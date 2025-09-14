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

impl HooksConfig {
    pub fn from_toml(t: &HooksToml) -> anyhow::Result<Self> {
        let map = |ht: &HookToml| -> anyhow::Result<HookSpec> {
            let cmd = ht
                .command
                .clone()
                .ok_or_else(|| anyhow::anyhow!("hook.command is required"))?;
            let route = match ht.route.as_deref() {
                Some("ui") | None => HookRoute::Ui,
                Some("llm") => HookRoute::Llm,
                Some("both") => HookRoute::Both,
                Some(other) => return Err(anyhow::anyhow!(format!("invalid route: {other}"))),
            };
            let on_error = match ht.on_error.as_deref() {
                Some("fail-closed") => HookOnError::FailClosed,
                Some("fail-open") | None => HookOnError::FailOpen,
                Some(other) => return Err(anyhow::anyhow!(format!("invalid on_error: {other}"))),
            };
            Ok(HookSpec {
                command: cmd,
                route,
                timeout_ms: ht.timeout_ms.unwrap_or(2000),
                on_error,
            })
        };

        Ok(Self {
            session_start: t.session_start.as_ref().map(&map).transpose()?,
            session_end: t.session_end.as_ref().map(&map).transpose()?,
            pre_command: t.pre_command.as_ref().map(&map).transpose()?,
            post_command: t.post_command.as_ref().map(&map).transpose()?,
        })
    }
}
