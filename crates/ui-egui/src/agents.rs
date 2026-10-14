//! AI agents channel (spec: docs/superpowers/specs/2026-10-14-ai-agents-design.md): requests
//! from AI apps (Claude, Codex, Antigravity) through `wordcraft-cli mcp --agent`, served only
//! while "Allow AI agents" is on, their edits recorded as tracked changes by the agent.

use serde_json::json;

use crate::{ControlRequest, WordApp, control};

/// The answer to every agent request while "Allow AI agents" is off.
pub const TURNED_OFF: &str = "AI agents are turned off in WordCraft: turn them on with the AI agents button in its title bar";

/// The author of edits from an agent that didn't name itself.
const UNNAMED: &str = "AI agent";

impl WordApp {
    /// Serve AI agent requests from `rx` (the host's agents channel).
    pub fn with_agents(mut self, rx: std::sync::mpsc::Receiver<ControlRequest>) -> Self {
        self.agent_rx = Some(rx);
        self
    }

    /// Start the agents channel when "Allow AI agents" is on and it isn't running yet. Hosts call
    /// this after restoring preferences; switching the setting on calls it too.
    pub fn sync_agents(&mut self) {
        if self.ui.agents_enabled
            && self.agent_rx.is_none()
            && let Some(start) = &self.services.start_agents
        {
            self.agent_rx = start();
        }
    }

    pub(crate) fn drain_agents(&mut self, ctx: &egui::Context) {
        let Some(rx) = self.agent_rx.take() else { return };
        while let Ok(req) = rx.try_recv() {
            if !self.ui.agents_enabled {
                let _ = req.reply.send(json!({"ok": false, "error": TURNED_OFF}));
                continue;
            }
            let outcome = self.as_agent(req.agent.as_deref().unwrap_or(UNNAMED), |app| control::handle(app, ctx, &req));
            self.deliver(req.reply.clone(), outcome);
        }
        self.agent_rx = Some(rx);
    }

    /// Run `f` with the agent as the author and Track Changes on, then put the user's author and
    /// tracking setting back (whatever the agent did to them).
    fn as_agent<T>(&mut self, agent: &str, f: impl FnOnce(&mut Self) -> T) -> T {
        let (author, tracking) = (self.session.author.clone(), self.session.doc.settings.track_changes);
        self.session.author = agent.trim().chars().take(64).collect();
        if self.session.author.is_empty() {
            self.session.author = UNNAMED.into();
        }
        self.session.doc.settings.track_changes = true;
        let out = f(self);
        self.session.author = author;
        self.session.doc.settings.track_changes = tracking;
        out
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use crate::{ControlRequest, Services, WordApp};

    fn app() -> (WordApp, std::sync::mpsc::Sender<ControlRequest>) {
        let (tx, rx) = std::sync::mpsc::channel();
        let a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), Services::default()).with_agents(rx);
        (a, tx)
    }

    /// Send one agent request and serve it; the reply.
    fn ask(a: &mut WordApp, tx: &std::sync::mpsc::Sender<ControlRequest>, agent: Option<&str>, method: &str, params: Value) -> Value {
        let (mut req, reply) = ControlRequest::new(method, params);
        req.agent = agent.map(str::to_string);
        tx.send(req).unwrap();
        a.drain_agents(&egui::Context::default());
        reply.try_recv().unwrap()
    }

    fn insert(text: &str) -> Value {
        json!({"command": "text.insert", "params": {"text": text}})
    }

    #[test]
    fn agent_request_refused_when_off() {
        let (mut a, tx) = app();
        assert!(!a.ui.agents_enabled, "off by default");
        let before = serde_json::to_value(&a.session.doc).unwrap();
        let r = ask(&mut a, &tx, Some("Claude"), "engine.execute", insert("Hello"));
        assert_eq!(r["ok"], false);
        assert!(r["error"].as_str().unwrap().contains("AI agents are turned off"), "{r}");
        assert_eq!(serde_json::to_value(&a.session.doc).unwrap(), before, "nothing changed");
    }

    #[test]
    fn agent_edit_is_tracked_under_agent_name() {
        let (mut a, tx) = app();
        a.run("ui.aiAgents", json!({"value": true})).unwrap();
        let author = a.session.author.clone();
        let r = ask(&mut a, &tx, Some("Claude"), "engine.execute", insert("Hello from Claude"));
        assert_eq!(r["ok"], true, "{r}");
        let doc = serde_json::to_string(&a.session.doc).unwrap();
        assert!(doc.contains("Hello from Claude"), "the text went in");
        assert!(doc.contains("\"Claude\""), "a revision authored by Claude: {doc}");
        assert_eq!(a.session.author, author, "the user's name is back");
        assert!(!a.session.doc.settings.track_changes, "the user's tracking setting is back");
    }

    #[test]
    fn agent_without_a_name_is_ai_agent() {
        let (mut a, tx) = app();
        a.ui.agents_enabled = true;
        ask(&mut a, &tx, None, "engine.execute", insert("x"));
        assert!(serde_json::to_string(&a.session.doc).unwrap().contains("\"AI agent\""));
    }

    #[test]
    fn agent_edit_restores_user_settings() {
        let (mut a, tx) = app();
        a.ui.agents_enabled = true;
        a.run("file.setAuthor", json!({"name": "Raul"})).unwrap();
        a.run("review.trackChanges", json!({"value": true})).unwrap();
        ask(&mut a, &tx, Some("Codex"), "engine.execute", insert("y"));
        // An agent that turns tracking off or renames the author doesn't keep that either.
        ask(&mut a, &tx, Some("Codex"), "engine.execute", json!({"command": "review.trackChanges", "params": {"value": false}}));
        ask(&mut a, &tx, Some("Codex"), "engine.execute", json!({"command": "file.setAuthor", "params": {"name": "Mallory"}}));
        assert_eq!(a.session.author, "Raul");
        assert!(a.session.doc.settings.track_changes);
        assert!(serde_json::to_string(&a.session.doc).unwrap().contains("\"Codex\""));
    }

    #[test]
    fn ai_agents_setting_persists_and_starts_the_channel() {
        let started = std::rc::Rc::new(std::cell::Cell::new(0));
        let s2 = started.clone();
        let services = Services {
            start_agents: Some(Box::new(move || {
                s2.set(s2.get() + 1);
                Some(std::sync::mpsc::channel().1)
            })),
            ..Default::default()
        };
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), services);
        assert_eq!(a.run("ui.aiAgents", json!({})).unwrap(), json!({"aiAgents": false}));
        assert_eq!(started.get(), 0, "nothing listens until switched on");
        assert_eq!(a.run("ui.aiAgents", json!({"value": true})).unwrap(), json!({"aiAgents": true}));
        assert_eq!(started.get(), 1);
        a.run("ui.aiAgents", json!({"value": false})).unwrap();
        a.run("ui.aiAgents", json!({"value": true})).unwrap();
        assert_eq!(started.get(), 1, "started once per session");
        for junk in [json!({"value": "yes"}), json!({"value": 1})] {
            assert!(a.run("ui.aiAgents", junk).is_err());
        }
        let saved = serde_json::to_string(&a.prefs()).unwrap();
        assert!(saved.contains(r#""agentsEnabled":true"#), "{saved}");
    }
}
