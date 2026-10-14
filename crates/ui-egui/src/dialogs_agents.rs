//! The AI agents dialog (title-bar button): the "Allow AI agents" switch and, for each AI app on
//! this Mac, its connection to WordCraft (spec: docs/superpowers/specs/2026-10-14-ai-agents-design.md).

use egui::{RichText, Ui};
use serde_json::json;
use wordcraft_agents::{Client, Entry, Status};

use crate::WordApp;
use crate::theme::{Tokens, medium, regular, semibold};

/// One AI app: what it is and whether WordCraft is in its config.
#[derive(Clone, Debug)]
pub struct Row {
    pub client: Client,
    pub status: Status,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct AgentsDialog {
    /// The home folder the clients' configs live under (`None` in the browser).
    #[serde(skip)]
    home: Option<std::path::PathBuf>,
    /// The bundled `wordcraft-cli` the configs point at; `None` hides Connect.
    #[serde(skip)]
    cli: Option<String>,
    #[serde(skip)]
    pub rows: Vec<Row>,
    /// The last connect/disconnect result.
    pub message: String,
}

impl AgentsDialog {
    pub fn detect(home: Option<std::path::PathBuf>, cli: Option<&str>) -> Self {
        let rows = Client::ALL
            .into_iter()
            .map(|client| Row { client, status: home.as_deref().map(|h| wordcraft_agents::status(client, h)).unwrap_or_default() })
            .collect();
        Self { home, cli: cli.map(str::to_string), rows, message: String::new() }
    }

    fn refresh(&mut self) {
        let msg = std::mem::take(&mut self.message);
        *self = Self::detect(self.home.take(), self.cli.as_deref());
        self.message = msg;
    }
}

/// The home folder, on desktop.
fn home() -> Option<std::path::PathBuf> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::env::var_os("HOME").filter(|h| !h.is_empty()).map(std::path::PathBuf::from)
    }
    #[cfg(target_arch = "wasm32")]
    {
        None
    }
}

pub fn open(app: &WordApp) -> AgentsDialog {
    AgentsDialog::detect(home(), app.services.agent_cli.as_deref())
}

/// The dialog's body; true closes it.
pub fn body(app: &mut WordApp, ui: &mut Ui, d: &mut AgentsDialog) -> bool {
    let t = Tokens::get(ui.ctx());
    ui.set_width(520.0);
    let mut on = app.ui.agents_enabled;
    if ui.checkbox(&mut on, RichText::new(tl!("Allow AI agents")).font(semibold(14.0))).changed() {
        let _ = app.run("ui.aiAgents", json!({"value": on}));
    }
    ui.label(
        RichText::new(tl!("AI apps on this Mac can read and edit the open document. Their edits arrive as tracked changes under their name."))
            .font(regular(12.0))
            .color(t.text_dim),
    );
    ui.add_space(12.0);
    let mut action: Option<(Client, bool)> = None;
    egui::Grid::new("ai_agents").num_columns(3).spacing(egui::vec2(18.0, 10.0)).show(ui, |ui| {
        for row in &d.rows {
            ui.label(RichText::new(tl!(row.client.label())).font(medium(13.5)));
            let (text, color) = match (row.status.installed, row.status.connected) {
                (_, true) => (tl!("Connected"), t.green),
                (true, false) => (tl!("Not connected"), t.text_dim),
                (false, false) => (tl!("Not installed"), t.text_disabled),
            };
            ui.label(RichText::new(text).font(regular(12.5)).color(color));
            if d.cli.is_some() && row.status.installed {
                let label = if row.status.connected { tl!("Disconnect") } else { tl!("Connect") };
                if ui.button(label).clicked() {
                    action = Some((row.client, !row.status.connected));
                }
            } else {
                ui.label("");
            }
            ui.end_row();
        }
    });
    if let (Some((client, connect)), Some(home), Some(cli)) = (action, d.home.clone(), d.cli.clone()) {
        let result = if connect {
            wordcraft_agents::connect(client, &home, &Entry::for_client(&cli, client))
        } else {
            wordcraft_agents::disconnect(client, &home)
        };
        d.message = match result {
            Ok(()) if connect => tl!("Restart {app} to load WordCraft.").replace("{app}", tl!(client.label())),
            Ok(()) => tl!("Restart {app} to drop WordCraft.").replace("{app}", tl!(client.label())),
            Err(e) => e.to_string(),
        };
        if connect && !app.ui.agents_enabled {
            let _ = app.run("ui.aiAgents", json!({"value": true}));
        }
        d.refresh();
    }
    if !d.message.is_empty() {
        ui.add_space(8.0);
        ui.label(RichText::new(d.message.as_str()).font(regular(12.5)).color(t.accent_text));
    }
    ui.add_space(10.0);
    ui.label(
        RichText::new(tl!("Then ask your AI app, for example: \u{201c}In WordCraft, proofread the open document.\u{201d}"))
            .font(regular(12.0))
            .color(t.text_dim),
    );
    ui.add_space(8.0);
    let mut close = ui.input(|i| i.key_pressed(egui::Key::Escape));
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        close |= ui.add(crate::widgets::primary_button(ui, tl!("Close"))).clicked();
    });
    close
}

#[cfg(test)]
mod tests {
    use egui_kittest::kittest::Queryable;
    use serde_json::json;

    use crate::{Services, WordApp};

    fn harness(a: WordApp) -> egui_kittest::Harness<'static, WordApp> {
        egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            a,
        )
    }

    #[test]
    fn ai_agents_dialog_lists_clients_and_switches() {
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), Services::default());
        a.run("ui.aiAgents", json!({"open": true})).unwrap();
        let mut h = harness(a);
        h.run_steps(4);
        for l in ["Claude", "Claude Code", "Codex (ChatGPT)", "Antigravity", "Allow AI agents"] {
            assert!(h.query_by_label(l).is_some(), "{l}");
        }
        assert!(h.query_by_label("Connect").is_none(), "no Connect without a bundled wordcraft-cli");
        h.get_by_label("Allow AI agents").click();
        h.run_steps(3);
        assert!(h.state().ui.agents_enabled);
    }

    #[test]
    fn title_bar_button_opens_the_dialog() {
        let a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), Services::default());
        let mut h = harness(a);
        h.run_steps(4);
        h.get_by_label("AI agents").click();
        h.run_steps(4);
        assert!(h.query_by_label("Allow AI agents").is_some());
    }

    #[test]
    fn connect_buttons_with_a_bundled_cli() {
        let services = Services { agent_cli: Some("/Applications/WordCraft.app/Contents/MacOS/wordcraft-cli".into()), ..Default::default() };
        let mut d = super::AgentsDialog::detect(None, services.agent_cli.as_deref());
        assert_eq!(d.rows.len(), 4);
        assert!(d.rows.iter().all(|r| !r.status.installed), "no home folder: nothing found");
        d.message = "x".into();
        assert_eq!(d.message, "x");
    }
}
