# AI agents (MCP connect): design

Status: approved in chat 2026-10-14. Sub-project 3a of 3 (after the start page and icons). The
in-app chat pane with API keys is a later sub-project (3b).

## Goal

Let Raul's own AI apps — Claude (desktop app and Claude Code), Codex (ChatGPT account) and
Antigravity (Google account) — read and edit the document open in WordCraft through the existing
MCP server, using his subscriptions, with one click to connect and every agent edit arriving as a
tracked change under the agent's name.

Success: with "Allow AI agents" on and Claude connected, asking Claude to change the open
document produces tracked changes authored "Claude" that Accept/Reject handle; with the switch
off, no request is served; connecting/disconnecting touches only the `wordcraft` entry of each
client's config and keeps a backup.

## Components

1. **`crates/agents`** (new, layer 5, no egui): the supported clients and their config files.
   - `Client` enum: `ClaudeDesktop`, `ClaudeCode`, `Codex`, `Antigravity`; `label()`, `agent_name()`
     ("Claude", "Claude", "Codex", "Antigravity"), `config_path(home)`:
     `~/Library/Application Support/Claude/claude_desktop_config.json`, `~/.claude.json`,
     `~/.codex/config.toml`, `~/.gemini/antigravity/mcp_config.json`.
   - `Entry { command: String, args: Vec<String> }` — the server line: `<cli> mcp --connect
     127.0.0.1:<port> --agent <name>`.
   - `status(client, home) -> Status { installed, connected }`: installed = config file or its
     folder exists; connected = a `wordcraft` server entry exists.
   - `connect(client, home, entry)` / `disconnect(client, home)`: JSON clients edit
     `mcpServers.wordcraft` (creating the object/file when missing); Codex edits the
     `[mcp_servers.wordcraft]` table in TOML text (replace an existing table block, else append),
     leaving every other byte as it was. Before writing, the old file is copied to
     `<file>.wordcraft-backup`. Writes go to a temp file then rename. A file that doesn't parse is
     an error and is never overwritten.
   - Pure functions over strings (`json_set`, `json_remove`, `toml_set`, `toml_remove`) carry the
     logic and the tests; the fs wrappers are thin.
2. **CLI**: `wordcraft-cli mcp --connect ADDR --agent NAME` adds `"agent": NAME` to every request
   line (`Remote::with_agent`). `--agent` without `--connect` is an error.
3. **App control channel for agents** (`apps/wordcraft`): a second control server on port
   **7979** (`AGENTS_PORT`), started when "Allow AI agents" is on (at launch if the saved setting is
   on, or when switched on) through a new `Services.start_agents` hook; once started it stays
   bound for the session. Its key file is removed on exit like the `--control` one.
   `ControlRequest` gains `agent: Option<String>` (from the request's top-level `agent`).
4. **UI** (`crates/ui-egui`):
   - `UiState.agents_enabled: bool` (saved). Command `ui.aiAgents` (`{"value": bool}` sets,
     no value reads; `{"open": true}` opens the dialog) — programmatic calls never open dialogs
     unless `open` is given by the UI.
   - Agent requests are drained from their own receiver. Switch off → every request answers
     `{"ok": false, "error": "AI agents are turned off in WordCraft (AI agents button)"}`.
   - While an agent request runs, the session's author is the request's agent name (fallback
     "AI agent") and track changes is on; both are restored afterwards. UI-input tools
     (`ui.click`, `ui.key`) queue events that run later under the user's settings (documented).
   - Title-bar button "AI agents" (icon `aiAgents`, purple family, original art: a spark over a
     chat bubble) beside Comments; opens the dialog: the switch, one row per client (label,
     status: Not installed / Not connected / Connected, button Connect/Disconnect, hint "Restart
     <app> to load it"), and a usage hint. Buttons hidden when `Services.agent_cli` is `None`
     (web, tests without a CLI).
   - `Services.agent_cli: Option<String>` — path to the bundled `wordcraft-cli`.
5. **Packaging**: `install-local.sh` builds `wordcraft-cli` too and copies it to
   `WordCraft.app/Contents/MacOS/wordcraft-cli`; the app finds it next to its own executable.

## Never-crash and safety

No panics; malformed client configs are reported, not rewritten; only loopback; the agents
channel uses the same key mechanism as `--control`; nothing listens while the switch has never
been on in this session.

## Testing

- `agents`: unit tests for each format — missing file, empty file, invalid JSON/TOML (error,
  untouched), other servers kept, idempotent connect, disconnect when absent, backup written,
  Codex table replaced in the middle of a file.
- CLI/mcp: `Remote` request lines carry `agent` when set.
- ui-egui: `ui.aiAgents` set/read/persist; agent request with switch off is refused; agent
  `engine.execute` of `text.insert` produces a tracked insertion authored by the agent and leaves
  the user's author and tracking unchanged; dialog shows the four clients.
- Manual end-to-end: reinstall, turn on, connect Claude Code, ask it to insert a sentence; see the
  tracked change.
- `cargo xtask ci` (known upstream Turkish proofing failure excepted).
