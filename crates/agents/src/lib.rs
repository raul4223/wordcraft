//! AI agents: the AI apps that can drive WordCraft over MCP, and the `wordcraft` server entry in
//! their config files (spec: docs/superpowers/specs/2026-10-14-ai-agents-design.md).

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use std::path::{Path, PathBuf};

use serde_json::{Map, Value, json};

/// The loopback port the app's AI agents channel listens on.
pub const AGENTS_PORT: u16 = 7979;

/// The server name written into every client's config.
pub const ENTRY_NAME: &str = "wordcraft";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{path} is not valid JSON ({reason}); it was left unchanged")]
    InvalidJson { path: String, reason: String },
    #[error("{0}")]
    Shape(String),
    #[error("{path}: {source}")]
    Io { path: String, source: std::io::Error },
}

/// An AI app that can drive WordCraft over MCP.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Client {
    ClaudeDesktop,
    ClaudeCode,
    Codex,
    Antigravity,
}

impl Client {
    pub const ALL: [Client; 4] = [Client::ClaudeDesktop, Client::ClaudeCode, Client::Codex, Client::Antigravity];

    /// Name shown in the AI agents dialog.
    pub fn label(self) -> &'static str {
        match self {
            Client::ClaudeDesktop => "Claude",
            Client::ClaudeCode => "Claude Code",
            Client::Codex => "Codex (ChatGPT)",
            Client::Antigravity => "Antigravity",
        }
    }

    /// The author name its tracked changes carry.
    pub fn agent_name(self) -> &'static str {
        match self {
            Client::ClaudeDesktop | Client::ClaudeCode => "Claude",
            Client::Codex => "Codex",
            Client::Antigravity => "Antigravity",
        }
    }

    /// The client's MCP config file under the home folder `home`.
    pub fn config_path(self, home: &Path) -> PathBuf {
        match self {
            Client::ClaudeDesktop => home.join("Library/Application Support/Claude/claude_desktop_config.json"),
            Client::ClaudeCode => home.join(".claude.json"),
            Client::Codex => home.join(".codex/config.toml"),
            Client::Antigravity => home.join(".gemini/antigravity/mcp_config.json"),
        }
    }

    fn is_toml(self) -> bool {
        self == Client::Codex
    }
}

/// The MCP server line a client runs: `command args…`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub command: String,
    pub args: Vec<String>,
}

impl Entry {
    /// `<cli> mcp --connect 127.0.0.1:<AGENTS_PORT> --agent <name>`.
    pub fn for_client(cli: &str, client: Client) -> Self {
        let args = ["mcp", "--connect", &format!("127.0.0.1:{AGENTS_PORT}"), "--agent", client.agent_name()].map(str::to_string).to_vec();
        Entry { command: cli.to_string(), args }
    }
}

/// Whether a client is on this machine and whether WordCraft is in its config.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Status {
    pub installed: bool,
    pub connected: bool,
}

pub fn status(client: Client, home: &Path) -> Status {
    let p = client.config_path(home);
    let installed = p.exists() || p.parent().is_some_and(Path::exists);
    let connected = std::fs::read_to_string(&p).ok().is_some_and(|s| if client.is_toml() { toml_has(&s) } else { json_has(&s).unwrap_or(false) });
    Status { installed, connected }
}

/// Add (or update) WordCraft's server entry in the client's config.
pub fn connect(client: Client, home: &Path, entry: &Entry) -> Result<(), Error> {
    edit(client, home, |s| if client.is_toml() { Ok(toml_set(s, entry)) } else { json_set(s, entry) })
}

/// Remove WordCraft's server entry from the client's config; the rest stays.
pub fn disconnect(client: Client, home: &Path) -> Result<(), Error> {
    edit(client, home, |s| if client.is_toml() { Ok(toml_remove(s)) } else { json_remove(s) })
}

/// Where the copy of a config made before editing it goes.
pub fn backup_path(p: &Path) -> PathBuf {
    let mut s = p.as_os_str().to_os_string();
    s.push(".wordcraft-backup");
    PathBuf::from(s)
}

fn edit(client: Client, home: &Path, change: impl FnOnce(&str) -> Result<String, Error>) -> Result<(), Error> {
    let p = client.config_path(home);
    let io = |source| Error::Io { path: p.display().to_string(), source };
    let old = match std::fs::read_to_string(&p) {
        Ok(s) => Some(s),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(io(e)),
    };
    let new = change(old.as_deref().unwrap_or("")).map_err(|e| match e {
        Error::InvalidJson { reason, .. } => Error::InvalidJson { path: p.display().to_string(), reason },
        other => other,
    })?;
    if old.as_deref() == Some(new.as_str()) {
        return Ok(());
    }
    if let Some(dir) = p.parent() {
        std::fs::create_dir_all(dir).map_err(io)?;
    }
    if let Some(old) = &old {
        std::fs::write(backup_path(&p), old).map_err(io)?;
    }
    let mut tmp = p.as_os_str().to_os_string();
    tmp.push(".wordcraft-tmp");
    let tmp = PathBuf::from(tmp);
    std::fs::write(&tmp, new).map_err(io)?;
    std::fs::rename(&tmp, &p).map_err(io)
}

fn parse_object(src: &str) -> Result<Map<String, Value>, Error> {
    if src.trim().is_empty() {
        return Ok(Map::new());
    }
    match serde_json::from_str::<Value>(src) {
        Ok(Value::Object(m)) => Ok(m),
        Ok(_) => Err(Error::Shape("the config file is not a JSON object".into())),
        Err(e) => Err(Error::InvalidJson { path: String::new(), reason: e.to_string() }),
    }
}

fn servers(m: &mut Map<String, Value>) -> Result<&mut Map<String, Value>, Error> {
    match m.entry("mcpServers").or_insert_with(|| json!({})) {
        Value::Object(s) => Ok(s),
        _ => Err(Error::Shape("`mcpServers` is not an object".into())),
    }
}

fn to_text(m: Map<String, Value>) -> String {
    let mut s = serde_json::to_string_pretty(&Value::Object(m)).unwrap_or_default();
    s.push('\n');
    s
}

/// The config text with `mcpServers.wordcraft` set to `entry`.
pub fn json_set(src: &str, entry: &Entry) -> Result<String, Error> {
    let mut m = parse_object(src)?;
    servers(&mut m)?.insert(ENTRY_NAME.into(), json!({"command": entry.command, "args": entry.args}));
    Ok(to_text(m))
}

/// The config text without `mcpServers.wordcraft`.
pub fn json_remove(src: &str) -> Result<String, Error> {
    let mut m = parse_object(src)?;
    if let Some(Value::Object(s)) = m.get_mut("mcpServers") {
        s.remove(ENTRY_NAME);
    }
    Ok(to_text(m))
}

pub fn json_has(src: &str) -> Result<bool, Error> {
    Ok(parse_object(src)?.get("mcpServers").and_then(|s| s.get(ENTRY_NAME)).is_some())
}

const TOML_HEADER: &str = "[mcp_servers.wordcraft]";

/// Whether a TOML line starts WordCraft's table or one of its subtables.
fn is_ours(line: &str) -> bool {
    let t = line.trim();
    t == TOML_HEADER || t.starts_with("[mcp_servers.wordcraft.")
}

fn is_header(line: &str) -> bool {
    line.trim_start().starts_with('[')
}

fn toml_str(s: &str) -> String {
    let mut out = String::from("\"");
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            c if c.is_control() => out.push_str(&format!("\\u{:04X}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn toml_block(entry: &Entry) -> String {
    let args: Vec<String> = entry.args.iter().map(|a| toml_str(a)).collect();
    format!("{TOML_HEADER}\ncommand = {}\nargs = [{}]\n", toml_str(&entry.command), args.join(", "))
}

/// Codex config text with WordCraft's table (and its subtables) taken out. Lines are kept byte
/// for byte; blank lines right before the removed table go with it.
pub fn toml_remove(src: &str) -> String {
    let mut out: Vec<&str> = Vec::new();
    let mut skipping = false;
    for line in src.split_inclusive('\n') {
        if is_header(line) {
            skipping = is_ours(line);
            if skipping {
                while out.last().is_some_and(|l| l.trim().is_empty()) {
                    out.pop();
                }
                continue;
            }
        }
        if !skipping {
            out.push(line);
        }
    }
    let mut s: String = out.concat();
    if !src.is_empty() && !s.is_empty() && !s.ends_with('\n') {
        s.push('\n');
    }
    s
}

/// Codex config text with WordCraft's table set to `entry` (appended at the end).
pub fn toml_set(src: &str, entry: &Entry) -> String {
    let mut s = toml_remove(src);
    if !s.is_empty() {
        if !s.ends_with('\n') {
            s.push('\n');
        }
        s.push('\n');
    }
    s.push_str(&toml_block(entry));
    s
}

pub fn toml_has(src: &str) -> bool {
    src.lines().any(|l| l.trim() == TOML_HEADER)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry() -> Entry {
        Entry {
            command: "/Applications/WordCraft.app/Contents/MacOS/wordcraft-cli".into(),
            args: vec!["mcp".into(), "--connect".into(), "127.0.0.1:7979".into(), "--agent".into(), "Claude".into()],
        }
    }

    #[test]
    fn json_set_creates_from_empty_and_missing() {
        for src in ["", "  \n"] {
            let out = json_set(src, &entry()).unwrap();
            let v: serde_json::Value = serde_json::from_str(&out).unwrap();
            assert_eq!(v["mcpServers"]["wordcraft"]["command"], entry().command);
            assert_eq!(v["mcpServers"]["wordcraft"]["args"][4], "Claude");
        }
    }

    #[test]
    fn json_set_keeps_other_keys() {
        let src = r#"{"numStartups": 42, "projects": {"/a": {"x": 1}}, "mcpServers": {"aws-api": {"command": "uvx"}}}"#;
        let v: serde_json::Value = serde_json::from_str(&json_set(src, &entry()).unwrap()).unwrap();
        assert_eq!(v["numStartups"], 42);
        assert_eq!(v["projects"]["/a"]["x"], 1);
        assert_eq!(v["mcpServers"]["aws-api"]["command"], "uvx");
        assert!(v["mcpServers"]["wordcraft"].is_object());
    }

    #[test]
    fn json_set_is_idempotent_and_remove_undoes() {
        let once = json_set("{}", &entry()).unwrap();
        assert_eq!(json_set(&once, &entry()).unwrap(), once);
        let v: serde_json::Value = serde_json::from_str(&json_remove(&once).unwrap()).unwrap();
        assert!(v["mcpServers"].get("wordcraft").is_none());
        assert_eq!(json_remove("{}").unwrap(), json_remove("{}").unwrap());
    }

    #[test]
    fn json_rejects_invalid_and_non_objects() {
        assert!(json_set("{not json", &entry()).is_err());
        assert!(json_set("[1,2]", &entry()).is_err());
        assert!(json_set(r#"{"mcpServers": 3}"#, &entry()).is_err());
        assert!(json_remove("{oops").is_err());
    }

    #[test]
    fn json_has_entry() {
        assert!(!json_has("").unwrap());
        assert!(json_has(&json_set("{}", &entry()).unwrap()).unwrap());
    }

    const CODEX: &str = "model = \"gpt-5\"\n\n[mcp_servers.node_repl]\ncommand = \"node\"\n\n[mcp_servers.wordcraft]\ncommand = \"old\"\nargs = []\n\n[mcp_servers.wordcraft.env]\nX = \"1\"\n\n[projects.\"/a\"]\ntrust_level = \"trusted\"\n";

    #[test]
    fn toml_set_appends_when_missing() {
        let out = toml_set("model = \"gpt-5\"\n", &entry());
        assert!(out.starts_with("model = \"gpt-5\"\n"));
        assert!(out.contains("[mcp_servers.wordcraft]\ncommand = \"/Applications/WordCraft.app/Contents/MacOS/wordcraft-cli\"\n"));
        assert!(out.contains("args = [\"mcp\", \"--connect\", \"127.0.0.1:7979\", \"--agent\", \"Claude\"]\n"));
        assert!(toml_has(&out));
        assert!(toml_set("", &entry()).starts_with("[mcp_servers.wordcraft]\n"));
    }

    #[test]
    fn toml_set_replaces_in_the_middle_and_keeps_neighbours() {
        let out = toml_set(CODEX, &entry());
        assert!(!out.contains("command = \"old\""));
        assert!(!out.contains("[mcp_servers.wordcraft.env]"));
        assert!(out.contains("[mcp_servers.node_repl]\ncommand = \"node\"\n"));
        assert!(out.contains("[projects.\"/a\"]\ntrust_level = \"trusted\"\n"));
        assert_eq!(out.matches("[mcp_servers.wordcraft]").count(), 1);
        assert_eq!(toml_set(&out, &entry()), out, "idempotent");
    }

    #[test]
    fn toml_remove_drops_subtables() {
        let out = toml_remove(CODEX);
        assert!(!toml_has(&out));
        assert!(!out.contains("wordcraft"));
        assert!(out.contains("[mcp_servers.node_repl]") && out.contains("[projects.\"/a\"]") && out.starts_with("model = \"gpt-5\""));
        assert_eq!(toml_remove("model = 1\n"), "model = 1\n");
    }

    #[test]
    fn toml_escapes_strings() {
        let e = Entry { command: "/tmp/my \"odd\" dir\\cli".into(), args: vec![] };
        assert!(toml_set("", &e).contains(r#"command = "/tmp/my \"odd\" dir\\cli""#));
    }

    #[test]
    fn toml_ignores_a_similarly_named_table() {
        let src = "[mcp_servers.wordcraft2]\ncommand = \"x\"\n";
        assert!(!toml_has(src));
        assert_eq!(toml_remove(src), src);
    }

    fn temp_home(tag: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("wordcraft-agents-test-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn connect_writes_backup_and_disconnect_removes() {
        let home = temp_home("connect");
        for client in Client::ALL {
            let p = client.config_path(&home);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            let original = if client == Client::Codex { "model = \"x\"\n".to_string() } else { "{\"keep\": true}".to_string() };
            std::fs::write(&p, &original).unwrap();
            assert_eq!(status(client, &home), Status { installed: true, connected: false });
            connect(client, &home, &entry()).unwrap();
            assert_eq!(status(client, &home), Status { installed: true, connected: true }, "{client:?}");
            assert_eq!(std::fs::read_to_string(backup_path(&p)).unwrap(), original);
            disconnect(client, &home).unwrap();
            assert!(!status(client, &home).connected);
            let now = std::fs::read_to_string(&p).unwrap();
            assert!(now.contains("keep") || now.contains("model"), "{client:?}: {now}");
        }
        let _ = std::fs::remove_dir_all(&home);
    }

    #[test]
    fn missing_client_is_not_installed_and_invalid_config_is_untouched() {
        let home = temp_home("missing");
        assert_eq!(status(Client::ClaudeDesktop, &home), Status { installed: false, connected: false });
        let p = Client::Antigravity.config_path(&home);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, "{broken").unwrap();
        assert!(connect(Client::Antigravity, &home, &entry()).is_err());
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "{broken");
        let _ = std::fs::remove_dir_all(&home);
    }

    #[test]
    fn connect_creates_a_missing_file_when_the_app_folder_exists() {
        let home = temp_home("create");
        let p = Client::ClaudeDesktop.config_path(&home);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        assert!(status(Client::ClaudeDesktop, &home).installed);
        connect(Client::ClaudeDesktop, &home, &entry()).unwrap();
        assert!(status(Client::ClaudeDesktop, &home).connected);
        let _ = std::fs::remove_dir_all(&home);
    }

    #[test]
    fn names_and_paths() {
        assert_eq!(Client::Codex.agent_name(), "Codex");
        assert_eq!(Client::ClaudeCode.agent_name(), "Claude");
        assert!(Client::Codex.config_path(std::path::Path::new("/h")).ends_with(".codex/config.toml"));
        assert_eq!(Client::ALL.len(), 4);
    }
}
