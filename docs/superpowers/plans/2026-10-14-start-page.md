# Start Page Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn File › Home into an interactive start page: icon sidebar, local-time greeting,
search over templates and documents, animated template cards, Recent/Pinned tabs with row actions,
empty states and a drop hint.

**Architecture:** A new `crates/ui-egui/src/home.rs` owns the Home page and its pure helpers;
`backstage.rs` keeps navigation and delegates. Host-only abilities (local UTC offset, reveal a file
in the file manager) arrive as optional `Services` hooks so L6 stays process-free and wasm builds.

**Tech Stack:** Rust, egui 0.36 (eframe), egui_kittest for UI tests, `jiff` (local UTC offset,
desktop app only), `open` crate (already in `apps/wordcraft`).

**Spec:** `docs/superpowers/specs/2026-10-14-start-page-design.md`

## Global Constraints

- No `unwrap/expect/panic!/unreachable!/todo!/unimplemented!` or `[i]` indexing in non-test code; no `unsafe`.
- All artwork original, drawn with `icons::paint` / painter calls; colours from `theme::Tokens`.
- Nothing below L6 changes; `ui-egui` gains no process spawning; `cargo xtask wasm` stays green.
- UI strings go through `tl!`; new strings get a `pt-br.tsv` row (TAB-separated: empty context, English, translation).
- `cargo xtask ci` green before each commit; commit message carries a task id (`HOME.n: …`).
- Builds use `CARGO_TARGET_DIR=$TMPDIR/wordcraft-dev` and `PATH=/opt/homebrew/opt/rustup/bin:$PATH`; delete the dir at the end.

## Review Focus

- A recent path whose file was deleted/moved: row shows dimmed "File not found", open shows the existing error, no panic. (Task 1 test `missing_file_has_no_time`, Task 3 test `missing_recent_row_is_dimmed_not_fatal`.)
- File mtime in the future or before 1970 (clock skew, odd filesystems): formatter returns a plain date, never panics. (Task 1 test `future_and_ancient_times_format_as_dates`.)
- Search text with accents/upper case ("RELATÓRIO", "résumé"): matches case-insensitively without byte slicing. (Task 1 test `filter_is_case_insensitive_and_unicode_safe`.)
- Old prefs file without `pinned`: loads with no pins. (Task 1 test `old_prefs_without_pinned_load`.)
- Pinning a path, then removing it from Recent: it stays in Pinned. (Task 1 test `remove_recent_keeps_pin`.)

---

### Task 1: Pure helpers and pin state (`home.rs`)

**Files:**
- Create: `crates/ui-egui/src/home.rs`
- Modify: `crates/ui-egui/src/lib.rs` (add `pub mod home;`, `UiState.pinned`, `Services.utc_offset`, `Services.reveal`)

**Interfaces:**
- Produces:
  - `UiState.pinned: Vec<String>` (serde default)
  - `Services.utc_offset: Option<Box<dyn Fn() -> i32>>` (seconds east of UTC)
  - `Services.reveal: Option<Box<dyn Fn(&str)>>`
  - `home::filter_docs(query: &str, docs: &[String]) -> Vec<String>`
  - `home::Clock { y: i64, m: u32, d: u32, hh: u32, mm: u32, days: i64 }` and `home::clock(unix: i64, offset: i32) -> Clock`
  - `home::friendly_time(file: Option<Clock>, now: Clock) -> String`
  - `home::DocKind` (`Word, Text, Rich, Web, Pdf, Other`) and `home::kind_of(path: &str) -> DocKind`
  - `home::toggle_pin(ui: &mut UiState, path: &str)`, `home::remove_recent(ui: &mut UiState, path: &str)`
  - `home::greeting(hour: u32) -> &'static str`

- [ ] **Step 1: Write failing tests** (bottom of `home.rs`, `#[cfg(test)] mod tests`)

```rust
use super::*;
fn c(y: i64, m: u32, d: u32, hh: u32, mm: u32) -> Clock {
    let days = days_from_civil(y, m, d);
    Clock { y, m, d, hh, mm, days }
}
#[test]
fn friendly_time_today_yesterday_week_older() {
    let now = c(2026, 10, 14, 18, 0);
    assert_eq!(friendly_time(Some(c(2026, 10, 14, 9, 5)), now), "Today, 09:05");
    assert_eq!(friendly_time(Some(c(2026, 10, 13, 23, 59)), now), "Yesterday");
    assert_eq!(friendly_time(Some(c(2026, 10, 10, 8, 0)), now), "Saturday");
    assert_eq!(friendly_time(Some(c(2025, 1, 2, 8, 0)), now), "2025-01-02");
}
#[test]
fn future_and_ancient_times_format_as_dates() {
    let now = c(2026, 10, 14, 18, 0);
    assert_eq!(friendly_time(Some(c(2030, 5, 1, 8, 0)), now), "2030-05-01");
    assert_eq!(friendly_time(Some(clock(-86_400 * 400, 0)), now), "1968-11-27");
    assert_eq!(friendly_time(Some(clock(i64::MIN, i32::MIN)), now).is_empty(), false);
}
#[test]
fn missing_file_has_no_time() {
    assert_eq!(friendly_time(None, c(2026, 10, 14, 18, 0)), "");
}
#[test]
fn clock_applies_offset() {
    let t = clock(1_760_468_400, -3 * 3600); // 2025-10-14 19:00 UTC
    assert_eq!((t.y, t.m, t.d, t.hh, t.mm), (2025, 10, 14, 16, 0));
}
#[test]
fn filter_is_case_insensitive_and_unicode_safe() {
    let docs = vec!["/a/Relatório Final.docx".to_string(), "/b/notes.md".to_string()];
    assert_eq!(filter_docs("RELATÓRIO", &docs), vec![docs[0].clone()]);
    assert_eq!(filter_docs("  ", &docs), docs);
    assert_eq!(filter_docs("/b", &docs), vec![docs[1].clone()]);
    assert!(filter_docs("zzz", &docs).is_empty());
}
#[test]
fn kinds_by_extension() {
    assert_eq!(kind_of("x.DOCX"), DocKind::Word);
    assert_eq!(kind_of("x.md"), DocKind::Text);
    assert_eq!(kind_of("x.odt"), DocKind::Rich);
    assert_eq!(kind_of("x.html"), DocKind::Web);
    assert_eq!(kind_of("x.pdf"), DocKind::Pdf);
    assert_eq!(kind_of("noext"), DocKind::Other);
}
#[test]
fn toggle_pin_adds_newest_first_and_removes() {
    let mut ui = crate::UiState::default();
    toggle_pin(&mut ui, "/a");
    toggle_pin(&mut ui, "/b");
    assert_eq!(ui.pinned, vec!["/b", "/a"]);
    toggle_pin(&mut ui, "/a");
    assert_eq!(ui.pinned, vec!["/b"]);
}
#[test]
fn remove_recent_keeps_pin() {
    let mut ui = crate::UiState::default();
    ui.recent = vec!["/a".into(), "/b".into()];
    toggle_pin(&mut ui, "/a");
    remove_recent(&mut ui, "/a");
    assert_eq!(ui.recent, vec!["/b"]);
    assert_eq!(ui.pinned, vec!["/a"]);
}
#[test]
fn old_prefs_without_pinned_load() {
    let ui: crate::UiState = serde_json::from_str(r#"{"recent":["/a"]}"#).unwrap();
    assert!(ui.pinned.is_empty());
    assert_eq!(ui.recent, vec!["/a"]);
}
#[test]
fn greeting_by_hour() {
    assert_eq!(greeting(5), "Good morning");
    assert_eq!(greeting(12), "Good afternoon");
    assert_eq!(greeting(18), "Good evening");
}
```

- [ ] **Step 2: Run, expect compile failure**

Run: `cargo test -p wordcraft-ui-egui home::tests`
Expected: FAIL (module/functions missing).

- [ ] **Step 3: Implement helpers**

```rust
//! File › Home: greeting, template row, search, Recent/Pinned documents (spec:
//! docs/superpowers/specs/2026-10-14-start-page-design.md).

/// A local wall-clock time, plus the day number (days since 1970-01-01) for day arithmetic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Clock { pub y: i64, pub m: u32, pub d: u32, pub hh: u32, pub mm: u32, pub days: i64 }

/// Unix seconds shifted by `offset` seconds east of UTC, as a local clock (saturating).
pub fn clock(unix: i64, offset: i32) -> Clock {
    let t = unix.saturating_add(i64::from(offset));
    let days = t.div_euclid(86_400);
    let secs = t.rem_euclid(86_400);
    let (y, m, d) = wordcraft_engine::cmd::civil_from_days(days);
    Clock { y, m, d, hh: (secs / 3600) as u32, mm: ((secs % 3600) / 60) as u32, days }
}

/// "Today, 14:32", "Yesterday", a weekday within the last week, else `YYYY-MM-DD`; "" when unknown.
pub fn friendly_time(file: Option<Clock>, now: Clock) -> String {
    let Some(f) = file else { return String::new() };
    let ago = now.days.saturating_sub(f.days);
    match ago {
        0 => format!("{}, {:02}:{:02}", tl!("Today"), f.hh, f.mm),
        1 => tl!("Yesterday").to_string(),
        2..=6 => tl!(WEEKDAYS.get(f.days.rem_euclid(7) as usize).copied().unwrap_or("")).to_string(),
        _ => format!("{:04}-{:02}-{:02}", f.y, f.m, f.d),
    }
}
/// 1970-01-01 was a Thursday.
const WEEKDAYS: [&str; 7] = ["Thursday", "Friday", "Saturday", "Sunday", "Monday", "Tuesday", "Wednesday"];

pub fn filter_docs(query: &str, docs: &[String]) -> Vec<String> {
    let q = query.trim().to_lowercase();
    docs.iter().filter(|p| q.is_empty() || p.to_lowercase().contains(&q)).cloned().collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocKind { Word, Text, Rich, Web, Pdf, Other }

pub fn kind_of(path: &str) -> DocKind {
    let ext = std::path::Path::new(path).extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
    match ext.as_str() {
        "docx" | "doc" | "docm" | "dotx" | "dotm" => DocKind::Word,
        "md" | "markdown" | "txt" => DocKind::Text,
        "rtf" | "odt" | "fodt" => DocKind::Rich,
        "html" | "htm" | "mht" => DocKind::Web,
        "pdf" => DocKind::Pdf,
        _ => DocKind::Other,
    }
}

pub fn toggle_pin(ui: &mut crate::UiState, path: &str) {
    if ui.pinned.iter().any(|p| p == path) {
        ui.pinned.retain(|p| p != path);
    } else {
        ui.pinned.insert(0, path.to_string());
        ui.pinned.truncate(crate::MAX_RECENT);
    }
}

pub fn remove_recent(ui: &mut crate::UiState, path: &str) {
    ui.recent.retain(|p| p != path);
}

pub fn greeting(hour: u32) -> &'static str {
    if hour < 12 { "Good morning" } else if hour < 18 { "Good afternoon" } else { "Good evening" }
}
```

Plus a test-only `days_from_civil(y, m, d) -> i64` (inverse of `civil_from_days`, Hinnant) inside `mod tests`.
In `lib.rs`: `pub mod home;`; in `UiState` after `recent`: `/// Documents pinned on File › Home, newest pin first.` `pub pinned: Vec<String>,` and `pinned: Vec::new(),` in `Default`; make `MAX_RECENT` `pub(crate)` if it isn't; in `Services` add:

```rust
    /// Desktop: the local time zone's offset from UTC in seconds (east positive), for the Home
    /// greeting and document times. `None`: UTC.
    pub utc_offset: Option<Box<dyn Fn() -> i32>>,
    /// Desktop: show a file in the system file manager (File › Home › Show in Finder). `None`
    /// hides the action (web, tests).
    pub reveal: Option<Box<dyn Fn(&str)>>,
```

- [ ] **Step 4: Run tests, expect PASS**: `cargo test -p wordcraft-ui-egui home::tests`
- [ ] **Step 5: Commit** `HOME.1: Home helpers, pinned documents, utc_offset/reveal services`

### Task 2: Host hooks (desktop and web)

**Files:**
- Modify: `apps/wordcraft/Cargo.toml` (add `jiff = { version = "0.2", default-features = false, features = ["std", "tz-system", "tzdb-zoneinfo"] }`)
- Create: `apps/wordcraft/src/reveal.rs`
- Modify: `apps/wordcraft/src/main.rs` (`services()`: `utc_offset`, `reveal`)
- Modify: `apps/wordcraft-web/src/*` where `Services { … }` is built: `utc_offset` from `js_sys::Date::new_0().get_timezone_offset()` (minutes **west**, so `-(m * 60.0) as i32`).

**Interfaces:**
- Consumes: `Services.utc_offset`, `Services.reveal` from Task 1.

- [ ] **Step 1: Implement `reveal.rs`**

```rust
//! Show a file in the system file manager (File › Home › Show in Finder).

pub fn reveal(path: &str) {
    let p = std::path::Path::new(path);
    #[cfg(target_os = "macos")]
    let r = std::process::Command::new("open").arg("-R").arg(p).spawn().map(|_| ());
    #[cfg(target_os = "windows")]
    let r = std::process::Command::new("explorer").arg(format!("/select,{}", p.display())).spawn().map(|_| ());
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let r = open::that_detached(p.parent().unwrap_or(p));
    if let Err(e) = r {
        log::warn!("reveal {path}: {e}");
    }
}
```

- [ ] **Step 2: Wire services** in `main.rs`:

```rust
        utc_offset: Some(Box::new(|| jiff::Zoned::now().offset().seconds())),
        reveal: Some(Box::new(reveal::reveal)),
```
and `mod reveal;`.

- [ ] **Step 3: Build** `cargo build -p wordcraft` and `cargo xtask wasm`. Expected: both succeed.
- [ ] **Step 4: Commit** `HOME.2: desktop/web hooks for local time and Show in Finder`

### Task 3: Home page UI

**Files:**
- Modify: `crates/ui-egui/src/home.rs` (add `pub(crate) fn page(app: &mut WordApp, ui: &mut Ui)` and drawing helpers)
- Modify: `crates/ui-egui/src/backstage.rs` (`home_page` → `crate::home::page`; `template_tile` `pub(crate)` with hover lift; `TEMPLATES` `pub(crate)`; `open_list` stays for the Open page)
- Modify: `crates/ui-egui/src/lib.rs` (`WordApp::local_now() -> home::Clock` using `services.utc_offset` and `cmd::now_unix`)
- Test: tests module in `home.rs`, and update `backstage.rs::home_and_new_show_different_sections`

**Interfaces:**
- Consumes: Task 1 helpers, `backstage::{template_tile, TEMPLATES}`, `Services.reveal`.
- Produces: `home::page`, `WordApp::local_now()`.

Layout (top to bottom):
1. Row: `heading(greeting(local_now().hh))` left; search `TextEdit` (hint "Search documents and templates", width 320) right-aligned. Query stored in `ctx.data` under `Id::new("home_search")`.
2. Templates: `horizontal_wrapped`, `template_tile` for each `TEMPLATES` entry whose label/id matches (reuse `matching_templates`). Hidden entirely when query matches none. "More templates →" after.
3. Tab row: two selectable labels "Recent" / "Pinned" (state in `ctx.data` `Id::new("home_tab")`, `"recent"` default) with an accent underline on the active one; right side "Open file…" button → `app.run("ui.openFileDialog", json!({}))`.
4. Column header (dim 12 pt): Name · Folder · Modified.
5. Rows (44 pt high, `available_width().min(900.0)`): type badge (rounded rect 22×26 tinted by `DocKind`: Word `t.blue`, Text `t.icon`, Rich `t.orange`, Web `t.green`, Pdf `t.red`, Other `t.text_dim`, with the extension in 8 pt white text), name (medium 13.5), folder (regular 12, dim), modified (`friendly_time`), then on hover: pin button (icon `pin`, accent when pinned) and `more` button. Hover fill animated with `ctx.animate_bool_with_time(id, hovered, 0.12)` blending `t.hover` alpha. Missing file (`std::fs::metadata` error; on wasm always "unknown", never "missing"): name and folder use `t.text_disabled`, folder text `tl!("File not found")`.
6. Row click → `file.open` and close Backstage (as today). Right-click or `more` → `egui::Popup::context_menu`/`menu_button` with: Open; Pin/Unpin (`toggle_pin`); Show in Finder (only if `services.reveal` is `Some`, label "Show in Finder" on macOS, "Show in folder" elsewhere); Copy path (`ctx.copy_text`); Remove from list (`remove_recent`; only on the Recent tab).
7. Empty states as in the spec: Recent → drawn illustration (page outline 48×60 with folded corner in `t.border_strong`, three text lines, an accent four-point sparkle top-right) + buttons "New document" (`file.new` blank, close Backstage) and "Open file…"; Pinned → "Pin documents to keep them here."; search with no template and no document hits → "Nothing matches your search."

Template tile hover lift (in `template_tile`): `let lift = ui.ctx().animate_bool_with_time(resp.id.with("lift"), resp.hovered(), 0.12);` draw a shadow rect `r.translate(vec2(0.0, 3.0))` with `t.page_shadow` scaled by `lift`, and offset the card/thumbnail by `-2.0 * lift` in y. Blank template: after the thumbnail, paint `icons::paint(.., "plus", t.accent, t.accent)` 36×36 centred.

- [ ] **Step 1: Write failing UI tests** in `home.rs` tests (harness like `backstage.rs` tests):

```rust
fn harness(a: WordApp) -> egui_kittest::Harness<'static, WordApp> {
    egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
        |ui, app: &mut WordApp| { let ctx = ui.ctx().clone(); app.logic(&ctx); app.ui(ui); }, a)
}
fn app_with_recent(paths: &[&str]) -> WordApp {
    let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), crate::Services::default());
    a.ui.recent = paths.iter().map(|p| p.to_string()).collect();
    a.run("ui.backstage", serde_json::json!({"value": true, "page": "home"})).unwrap();
    a
}
#[test]
fn home_has_tabs_search_and_all_templates() {
    let mut h = harness(app_with_recent(&[]));
    h.run_steps(4);
    for l in ["Recent", "Pinned", "Report", "Blank document", "Open file…", "New document"] {
        assert!(h.query_by_label(l).is_some(), "{l}");
    }
}
#[test]
fn pin_from_menu_moves_to_pinned_tab() {
    let mut h = harness(app_with_recent(&["/nowhere/a.docx"]));
    h.run_steps(4);
    h.get_by_label("More actions for a.docx").click();
    h.run_steps(2);
    h.get_by_label("Pin").click();
    h.run_steps(2);
    assert_eq!(h.state().ui.pinned, vec!["/nowhere/a.docx"]);
    h.get_by_label("Pinned").click();
    h.run_steps(2);
    assert!(h.query_by_label("a.docx").is_some());
}
#[test]
fn remove_from_list_drops_recent() {
    let mut h = harness(app_with_recent(&["/nowhere/a.docx", "/nowhere/b.md"]));
    h.run_steps(4);
    h.get_by_label("More actions for a.docx").click();
    h.run_steps(2);
    h.get_by_label("Remove from list").click();
    h.run_steps(2);
    assert_eq!(h.state().ui.recent, vec!["/nowhere/b.md"]);
}
#[test]
fn search_filters_documents_and_templates() {
    let mut h = harness(app_with_recent(&["/nowhere/a.docx", "/nowhere/budget.md"]));
    h.run_steps(4);
    h.get_by_label("Search documents and templates").focus();
    h.get_by_label("Search documents and templates").type_text("budget");
    h.run_steps(3);
    assert!(h.query_by_label("budget.md").is_some());
    assert!(h.query_by_label("a.docx").is_none());
    assert!(h.query_by_label("Letter").is_none());
}
#[test]
fn missing_recent_row_is_dimmed_not_fatal() {
    let mut h = harness(app_with_recent(&["/definitely/not/here.docx"]));
    h.run_steps(4);
    assert!(h.query_by_label("File not found").is_some());
}
#[test]
fn show_in_finder_hidden_without_service() {
    let mut h = harness(app_with_recent(&["/nowhere/a.docx"]));
    h.run_steps(4);
    h.get_by_label("More actions for a.docx").click();
    h.run_steps(2);
    assert!(h.query_by_label("Show in Finder").is_none() && h.query_by_label("Show in folder").is_none());
}
```

Rows expose accessible labels: the row as a button labelled with the file name; the `⋯` button as `"More actions for {name}"` (via `widget_info`, built with `tl!("More actions for {name}").replace("{name}", name)`).

- [ ] **Step 2: Run, expect FAIL**: `cargo test -p wordcraft-ui-egui home::tests`
- [ ] **Step 3: Implement `page` per layout above**; in `backstage.rs` replace `home_page` body with `crate::home::page(app, ui)`; update `home_and_new_show_different_sections` (drop the "Report absent from Home" assertion; find the "More templates →" link, unchanged).
- [ ] **Step 4: Run `cargo test -p wordcraft-ui-egui`**, expect PASS (all backstage tests too).
- [ ] **Step 5: Commit** `HOME.3: interactive File › Home with search, Recent/Pinned and row actions`

### Task 4: Sidebar icons, drop hint, translations

**Files:**
- Modify: `crates/ui-egui/src/backstage.rs` (sidebar)
- Modify: `crates/ui-egui/src/home.rs` (drop hint)
- Modify: `crates/ui-egui/src/i18n/pt-br.tsv`

- [ ] **Step 1: Sidebar.** Change `PAGES` to `[(&str, &str, &str); 10]` with icon names: home `homeFolder`, new `plus`, open `open`, info `info`, save `save`, saveAs `save`, print `print`, export `share`, close `close`, options `docPrefs`. Draw the icon 18×18 at `r.min.x + 30.0` in white (alpha 110 when disabled), text moves to `r.min.x + 52.0`. Hover fill: `let k = ui.ctx().animate_bool_with_time(resp.id.with("hov"), on && resp.hovered(), 0.12);` fill `Color32::from_white_alpha((26.0 * k) as u8)` when not active; active keeps alpha 46 plus a 3 pt white bar at the left edge. Existing backstage tests must still pass (labels unchanged).
- [ ] **Step 2: Drop hint.** In `home::page`, `if ui.ctx().input(|i| !i.raw.hovered_files.is_empty())`, paint over the central panel rect (`ui.max_rect()`): `t.accent` 6% fill, dashed accent outline (`Shape::dashed_line` on the rect's 4 sides, 2 pt, dash 8 gap 6), centred text `tl!("Drop to open")` semibold 18. Opening is already handled in `lib.rs`.
- [ ] **Step 3: Translations.** Append to `pt-br.tsv`:

```
	Good morning	Bom dia
	Good afternoon	Boa tarde
	Good evening	Boa noite
	Search documents and templates	Pesquisar documentos e modelos
	Pinned	Fixados
	Open file…	Abrir arquivo…
	New document	Novo documento
	Name	Nome
	Folder	Pasta
	Modified	Modificado
	Today	Hoje
	Yesterday	Ontem
	Monday	Segunda-feira
	Tuesday	Terça-feira
	Wednesday	Quarta-feira
	Thursday	Quinta-feira
	Friday	Sexta-feira
	Saturday	Sábado
	Sunday	Domingo
	File not found	Arquivo não encontrado
	More actions for {name}	Mais ações para {name}
	Pin	Fixar
	Unpin	Desafixar
	Show in Finder	Mostrar no Finder
	Show in folder	Mostrar na pasta
	Copy path	Copiar caminho
	Remove from list	Remover da lista
	Pin documents to keep them here.	Fixe documentos para mantê-los aqui.
	Nothing matches your search.	Nada corresponde à sua pesquisa.
	Drop to open	Solte para abrir
```
(skip any source already present — check with `grep -P '^\tSOURCE\t'`; the i18n tests flag duplicates.)
- [ ] **Step 4: Run `cargo test -p wordcraft-ui-egui`** (includes i18n audit). Expected PASS.
- [ ] **Step 5: Commit** `HOME.4: Backstage sidebar icons, drop hint, pt-BR strings`

### Task 5: Verify, document, install

- [ ] **Step 1:** `ui_shot` Home empty (`{"empty": true, "startScreen": true}`) and with recents (control `engine.execute` can't set recents; add the recent paths through a script that opens two saved sample files: `file.saveAs` to scratchpad paths then `ui.backstage {"value":true,"page":"home"}`), read the PNGs, fix visual issues.
- [ ] **Step 2:** `cargo xtask ci` — all green.
- [ ] **Step 3:** Docs: add a line to `docs/ui-parity.md` (Backstage Home row) and the revision history; `ROADMAP.md` progress log line.
- [ ] **Step 4:** Commit `HOME.5: docs`, push to `origin main`.
- [ ] **Step 5:** `packaging/macos/install-local.sh`; `rm -rf $TMPDIR/wordcraft-dev`.
