//! File › Home: greeting, template row, search, Recent/Pinned documents (spec:
//! docs/superpowers/specs/2026-10-14-start-page-design.md).

/// A local wall-clock time, plus its day number (days since 1970-01-01) for day arithmetic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Clock {
    pub y: i64,
    pub m: u32,
    pub d: u32,
    pub hh: u32,
    pub mm: u32,
    pub days: i64,
}

/// Unix seconds shifted by `offset` seconds east of UTC, as a local clock. Saturates instead of
/// overflowing, so any file time is safe.
pub fn clock(unix: i64, offset: i32) -> Clock {
    let t = unix.saturating_add(i64::from(offset));
    let days = t.div_euclid(86_400);
    let secs = t.rem_euclid(86_400);
    let (y, m, d) = wordcraft_engine::cmd::civil_from_days(days);
    Clock { y, m, d, hh: (secs / 3600) as u32, mm: ((secs % 3600) / 60) as u32, days }
}

/// 1970-01-01 (day 0) was a Thursday.
const WEEKDAYS: [&str; 7] = ["Thursday", "Friday", "Saturday", "Sunday", "Monday", "Tuesday", "Wednesday"];

/// "Today, 14:32", "Yesterday", a weekday within the last week, else `YYYY-MM-DD` (also for times
/// in the future); empty when the time is unknown.
pub fn friendly_time(file: Option<Clock>, now: Clock) -> String {
    let Some(f) = file else { return String::new() };
    match now.days.saturating_sub(f.days) {
        0 => format!("{}, {:02}:{:02}", tl!("Today"), f.hh, f.mm),
        1 => tl!("Yesterday").to_string(),
        2..=6 => WEEKDAYS.get(f.days.rem_euclid(7) as usize).map(|w| tl!(w).to_string()).unwrap_or_default(),
        _ => format!("{:04}-{:02}-{:02}", f.y, f.m, f.d),
    }
}

/// The documents whose path (name or folder) contains `query`, ignoring case; all for a blank query.
pub fn filter_docs(query: &str, docs: &[String]) -> Vec<String> {
    let q = query.trim().to_lowercase();
    docs.iter().filter(|p| q.is_empty() || p.to_lowercase().contains(&q)).cloned().collect()
}

/// What kind of file a document is, for its badge colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocKind {
    Word,
    Text,
    Rich,
    Web,
    Pdf,
    Other,
}

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

/// Pin `path` (newest pin first) or unpin it if it is pinned.
pub fn toggle_pin(ui: &mut crate::UiState, path: &str) {
    if ui.pinned.iter().any(|p| p == path) {
        ui.pinned.retain(|p| p != path);
    } else {
        ui.pinned.insert(0, path.to_string());
        ui.pinned.truncate(crate::MAX_RECENT);
    }
}

/// Remove `path` from Recent; a pin stays.
pub fn remove_recent(ui: &mut crate::UiState, path: &str) {
    ui.recent.retain(|p| p != path);
}

/// The Home greeting for a local hour.
pub fn greeting(hour: u32) -> &'static str {
    if hour < 12 {
        "Good morning"
    } else if hour < 18 {
        "Good afternoon"
    } else {
        "Good evening"
    }
}

use egui::{Align2, Color32, Rect, Sense, Stroke, Ui, pos2, vec2};
use serde_json::json;

use crate::theme::{Tokens, medium, regular, semibold};
use crate::{WordApp, icons};

/// What a document row's menu asked for; applied after drawing.
enum RowAction {
    Open,
    TogglePin,
    Reveal,
    CopyPath,
    Remove,
}

/// File › Home.
pub(crate) fn page(app: &mut WordApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    drop_hint(ui, &t);
    let search_id = egui::Id::new("home_search");
    let mut query = ui.ctx().data(|d| d.get_temp::<String>(search_id)).unwrap_or_default();
    // Header, templates and list share one column, as wide as the five template cards.
    let width = ui.available_width().min(1170.0);
    ui.allocate_ui_with_layout(vec2(width, 40.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
        ui.label(egui::RichText::new(tl!(greeting(app.local_now().hh))).font(semibold(26.0)));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let edit = egui::TextEdit::singleline(&mut query)
                .hint_text(tl!("Search documents and templates"))
                .desired_width(320.0)
                .margin(egui::Margin { left: 28, right: 8, top: 6, bottom: 6 });
            let r = ui.add(edit);
            icons::paint(
                ui.painter(),
                Rect::from_center_size(pos2(r.rect.min.x + 15.0, r.rect.center().y), vec2(16.0, 16.0)),
                "search",
                t.text_dim,
                t.accent,
            );
            if r.changed() {
                ui.ctx().data_mut(|d| d.insert_temp(search_id, query.clone()));
            }
        });
    });
    ui.add_space(18.0);

    let templates = crate::backstage::matching_templates(&query);
    if !templates.is_empty() {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(18.0, 18.0);
            for (label, template) in &templates {
                crate::backstage::template_tile(ui, app, label, template);
            }
        });
        ui.add_space(8.0);
        let more = ui.add(egui::Label::new(egui::RichText::new(tl!("More templates →")).font(medium(13.5)).color(t.accent)).sense(Sense::click()));
        if more.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
            app.ui.backstage_page = "new".into();
        }
        ui.add_space(26.0);
    }

    // Tabs and Open file….
    let tab_id = egui::Id::new("home_tab");
    let mut pinned_tab = ui.ctx().data(|d| d.get_temp::<bool>(tab_id)).unwrap_or(false);
    ui.allocate_ui_with_layout(vec2(width, 32.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
        ui.spacing_mut().item_spacing.x = 22.0;
        for (label, is_pinned) in [("Recent", false), ("Pinned", true)] {
            let active = pinned_tab == is_pinned;
            let r = ui.add(
                egui::Label::new(egui::RichText::new(tl!(label)).font(if active { semibold(15.0) } else { medium(15.0) }).color(if active {
                    t.text
                } else {
                    t.text_dim
                }))
                .sense(Sense::click()),
            );
            let k = ui.ctx().animate_bool_with_time(r.id.with("tab"), active, 0.15);
            if k > 0.0 {
                let y = r.rect.bottom() + 4.0;
                let w = r.rect.width() * k;
                ui.painter().line_segment([pos2(r.rect.center().x - w / 2.0, y), pos2(r.rect.center().x + w / 2.0, y)], Stroke::new(2.5, t.accent));
            }
            if r.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                pinned_tab = is_pinned;
                ui.ctx().data_mut(|d| d.insert_temp(tab_id, pinned_tab));
            }
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(egui::RichText::new(tl!("Open file…")).font(medium(13.5))).clicked() {
                let _ = app.run("ui.openFileDialog", json!({}));
            }
        });
    });
    ui.add_space(8.0);
    let (line, _) = ui.allocate_exact_size(vec2(width, 1.0), Sense::hover());
    ui.painter().rect_filled(line, 0.0, t.border);
    ui.add_space(6.0);

    let source: Vec<String> = if pinned_tab { app.ui.pinned.clone() } else { app.ui.recent.iter().take(app.ui.recent_shown()).cloned().collect() };
    let docs = filter_docs(&query, &source);
    if docs.is_empty() {
        empty_state(app, ui, pinned_tab, source.is_empty(), templates.is_empty());
        return;
    }
    // Column headings.
    let (head, _) = ui.allocate_exact_size(vec2(width, 22.0), Sense::hover());
    for (label, x) in [("Name", 48.0), ("Folder", width * 0.44), ("Modified", width * 0.72)] {
        ui.painter().text(pos2(head.min.x + x, head.center().y), Align2::LEFT_CENTER, tl!(label), regular(12.0), t.text_dim);
    }
    let now = app.local_now();
    let offset = app.utc_offset();
    let mut action = None;
    for path in docs {
        if let Some(a) = doc_row(app, ui, &path, width, now, offset, pinned_tab) {
            action = Some((a, path));
        }
    }
    if let Some((a, path)) = action {
        match a {
            RowAction::Open => {
                let _ = app.run("file.open", json!({"path": path}));
                app.ui.backstage = false;
                app.canvas.want_focus = true;
            }
            RowAction::TogglePin => toggle_pin(&mut app.ui, &path),
            RowAction::Reveal => {
                if let Some(reveal) = &app.services.reveal {
                    reveal(&path);
                }
            }
            RowAction::CopyPath => ui.ctx().copy_text(path),
            RowAction::Remove => remove_recent(&mut app.ui, &path),
        }
    }
}

/// While files are dragged over the window: "Drop to open" over the visible page (dropping is
/// handled for the whole window in `WordApp::logic`).
fn drop_hint(ui: &Ui, t: &Tokens) {
    if ui.ctx().input(|i| i.raw.hovered_files.is_empty()) {
        return;
    }
    let r = ui.clip_rect().shrink(12.0);
    egui::Area::new(egui::Id::new("home_drop_hint")).order(egui::Order::Foreground).fixed_pos(r.min).interactable(false).show(ui.ctx(), |ui| {
        let (r, _) = ui.allocate_exact_size(r.size(), Sense::hover());
        let p = ui.painter();
        p.rect_filled(r, 12.0, t.ribbon.gamma_multiply(0.92));
        p.rect_filled(r, 12.0, t.accent.gamma_multiply(0.06));
        let s = Stroke::new(2.0, t.accent);
        for (a, b) in
            [(r.left_top(), r.right_top()), (r.right_top(), r.right_bottom()), (r.right_bottom(), r.left_bottom()), (r.left_bottom(), r.left_top())]
        {
            p.extend(egui::Shape::dashed_line(&[a, b], s, 8.0, 6.0));
        }
        icons::paint(p, Rect::from_center_size(r.center() - vec2(0.0, 34.0), vec2(40.0, 40.0)), "open", t.accent, t.accent);
        let label = Rect::from_center_size(r.center() + vec2(0.0, 12.0), vec2(r.width(), 30.0));
        ui.put(label, egui::Label::new(egui::RichText::new(tl!("Drop to open")).font(semibold(18.0)).color(t.accent)).selectable(false));
    });
}

/// The file's modification time as unix seconds; `Err` when it can't be read (missing file).
/// The browser has no file system: `Ok(None)`.
fn modified(path: &str) -> Result<Option<i64>, ()> {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = path;
        Ok(None)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let meta = std::fs::metadata(path).map_err(|_| ())?;
        Ok(meta.modified().ok().and_then(|m| m.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX)))
    }
}

/// A folder with the home folder shown as `~`.
fn short_folder(folder: &str) -> String {
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(rest) = std::env::var("HOME").ok().filter(|h| !h.is_empty()).and_then(|h| folder.strip_prefix(&h).map(str::to_string))
        && (rest.is_empty() || rest.starts_with('/'))
    {
        return format!("~{rest}");
    }
    folder.to_string()
}

fn kind_color(t: &Tokens, k: DocKind) -> Color32 {
    match k {
        DocKind::Word => t.blue,
        DocKind::Text => t.icon,
        DocKind::Rich => t.orange,
        DocKind::Web => t.green,
        DocKind::Pdf => t.red,
        DocKind::Other => t.text_dim,
    }
}

#[allow(clippy::too_many_arguments)]
fn doc_row(app: &WordApp, ui: &mut Ui, path: &str, width: f32, now: Clock, offset: i32, pinned_tab: bool) -> Option<RowAction> {
    let t = Tokens::get(ui.ctx());
    let p = std::path::Path::new(path);
    let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| path.to_string());
    let folder = p.parent().map(|f| short_folder(&f.to_string_lossy())).unwrap_or_default();
    let stamp = modified(path);
    let missing = stamp.is_err();
    let when = friendly_time(stamp.ok().flatten().map(|s| clock(s, offset)), now);
    let pinned = app.ui.pinned.iter().any(|x| x == path);

    let (r, row) = ui.allocate_exact_size(vec2(width, 46.0), Sense::click());
    row.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &name));
    // The row's actions are always there for keyboard and screen readers; they show on hover.
    let more_rect = Rect::from_center_size(pos2(r.max.x - 22.0, r.center().y), vec2(28.0, 28.0));
    let pin_rect = Rect::from_center_size(pos2(r.max.x - 56.0, r.center().y), vec2(28.0, 28.0));
    let more = ui.interact(more_rect, row.id.with("more"), Sense::click());
    let more_label = tl!("More actions for {name}").replace("{name}", &name);
    more.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &more_label));
    let pin = ui.interact(pin_rect, row.id.with("pin"), Sense::click());
    let pin_label = if pinned { tl!("Unpin") } else { tl!("Pin") };
    let pin_name = if pinned { tl!("Unpin {name}") } else { tl!("Pin {name}") }.replace("{name}", &name);
    pin.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &pin_name));

    let menu_open = egui::Popup::is_id_open(ui.ctx(), egui::Popup::default_response_id(&more));
    let hot = row.hovered() || more.hovered() || pin.hovered() || menu_open;
    let k = ui.ctx().animate_bool_with_time(row.id.with("hover"), hot, 0.12);
    if k > 0.0 {
        ui.painter().rect_filled(r, 6.0, t.hover.gamma_multiply(k));
    }
    // Type badge.
    let badge = Rect::from_center_size(pos2(r.min.x + 22.0, r.center().y), vec2(24.0, 30.0));
    let tint = if missing { t.text_disabled } else { kind_color(&t, kind_of(path)) };
    ui.painter().rect_filled(badge, 4.0, tint);
    let fold = 7.0;
    ui.painter().add(egui::Shape::convex_polygon(
        vec![pos2(badge.max.x - fold, badge.min.y), badge.right_top(), pos2(badge.max.x, badge.min.y + fold)],
        t.ribbon,
        Stroke::NONE,
    ));
    let ext = p.extension().map(|e| e.to_string_lossy().to_uppercase()).unwrap_or_default();
    let ext: String = ext.chars().take(4).collect();
    ui.painter().text(pos2(badge.center().x, badge.max.y - 9.0), Align2::CENTER_CENTER, ext, semibold(7.5), Color32::WHITE);

    let col = |x: f32, w: f32| Rect::from_min_size(pos2(r.min.x + x, r.min.y), vec2(w, r.height()));
    let name_col = col(48.0, width * 0.44 - 56.0);
    let folder_col = col(width * 0.44, width * 0.28 - 10.0);
    ui.painter().with_clip_rect(name_col).text(
        pos2(name_col.min.x, r.center().y),
        Align2::LEFT_CENTER,
        &name,
        medium(13.5),
        if missing { t.text_disabled } else { t.text },
    );
    if missing {
        ui.put(folder_col, egui::Label::new(egui::RichText::new(tl!("File not found")).font(regular(12.0)).color(t.text_disabled)).selectable(false));
    } else {
        ui.painter().with_clip_rect(folder_col).text(pos2(folder_col.min.x, r.center().y), Align2::LEFT_CENTER, &folder, regular(12.0), t.text_dim);
    }
    ui.painter().text(pos2(r.min.x + width * 0.72, r.center().y), Align2::LEFT_CENTER, &when, regular(12.0), t.text_dim);

    if hot || pinned {
        let c = if pinned {
            t.accent
        } else if pin.hovered() {
            t.text
        } else {
            t.icon
        };
        if pin.hovered() {
            ui.painter().rect_filled(pin_rect, 4.0, t.pressed);
        }
        icons::paint(ui.painter(), pin_rect.shrink(6.0), "pin", c, t.accent);
    }
    if hot {
        if more.hovered() || menu_open {
            ui.painter().rect_filled(more_rect, 4.0, t.pressed);
        }
        icons::paint(ui.painter(), more_rect.shrink(6.0), "more", t.icon, t.accent);
    }

    let mut action = None;
    let mut menu = |ui: &mut Ui| {
        if ui.button(tl!("Open")).clicked() {
            action = Some(RowAction::Open);
        }
        if ui.button(pin_label).clicked() {
            action = Some(RowAction::TogglePin);
        }
        if app.services.reveal.is_some() {
            let label = if cfg!(target_os = "macos") { "Show in Finder" } else { "Show in folder" };
            if ui.add_enabled(!missing, egui::Button::new(tl!(label))).clicked() {
                action = Some(RowAction::Reveal);
            }
        }
        if ui.button(tl!("Copy path")).clicked() {
            action = Some(RowAction::CopyPath);
        }
        if !pinned_tab && ui.button(tl!("Remove from list")).clicked() {
            action = Some(RowAction::Remove);
        }
    };
    egui::Popup::menu(&more).show(&mut menu);
    row.context_menu(&mut menu);
    if pin.clicked() {
        return Some(RowAction::TogglePin);
    }
    if action.is_none() && row.clicked() {
        return Some(RowAction::Open);
    }
    action
}

/// No documents to list: a drawn illustration and a way forward.
fn empty_state(app: &mut WordApp, ui: &mut Ui, pinned_tab: bool, nothing_at_all: bool, no_templates: bool) {
    let t = Tokens::get(ui.ctx());
    ui.add_space(18.0);
    if !nothing_at_all {
        if no_templates {
            ui.label(egui::RichText::new(tl!("Nothing matches your search.")).color(t.text_dim));
        }
        return;
    }
    if pinned_tab {
        ui.horizontal(|ui| {
            let (r, _) = ui.allocate_exact_size(vec2(28.0, 28.0), Sense::hover());
            icons::paint(ui.painter(), r, "pin", t.text_dim, t.accent);
            ui.label(egui::RichText::new(tl!("Pin documents to keep them here.")).color(t.text_dim));
        });
        return;
    }
    ui.horizontal(|ui| {
        let (r, _) = ui.allocate_exact_size(vec2(84.0, 84.0), Sense::hover());
        illustration(ui.painter(), r, &t);
        ui.add_space(12.0);
        ui.vertical(|ui| {
            ui.add_space(14.0);
            ui.label(egui::RichText::new(tl!("Documents you open will show up here.")).font(medium(14.0)).color(t.text));
            ui.add_space(10.0);
            let new = egui::Button::new(egui::RichText::new(tl!("New document")).font(medium(13.5)).color(t.on_accent)).fill(t.accent);
            if ui.add(new).clicked() {
                let _ = app.run("file.new", json!({"template": "blank"}));
                app.ui.backstage = false;
                app.canvas.want_focus = true;
            }
        });
    });
}

/// A page with a few lines of text and a sparkle (original artwork).
fn illustration(p: &egui::Painter, r: Rect, t: &Tokens) {
    let page = Rect::from_center_size(r.center() + vec2(-4.0, 4.0), vec2(50.0, 64.0));
    p.rect_filled(page.translate(vec2(3.0, 3.0)), 4.0, t.page_shadow);
    p.rect(page, 4.0, Color32::WHITE, Stroke::new(1.5, t.border_strong), egui::StrokeKind::Inside);
    for (i, w) in [30.0, 24.0, 32.0, 18.0].into_iter().enumerate() {
        let y = page.min.y + 16.0 + i as f32 * 10.0;
        p.line_segment([pos2(page.min.x + 9.0, y), pos2(page.min.x + 9.0 + w, y)], Stroke::new(2.5, t.border_strong));
    }
    let c = pos2(page.max.x + 2.0, page.min.y + 2.0);
    let (a, b) = (11.0, 3.0);
    p.add(egui::Shape::convex_polygon(
        vec![
            pos2(c.x, c.y - a),
            pos2(c.x + b, c.y - b),
            pos2(c.x + a, c.y),
            pos2(c.x + b, c.y + b),
            pos2(c.x, c.y + a),
            pos2(c.x - b, c.y + b),
            pos2(c.x - a, c.y),
            pos2(c.x - b, c.y - b),
        ],
        t.accent,
        Stroke::NONE,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Days since 1970-01-01 for a civil date (inverse of `civil_from_days`, Howard Hinnant).
    fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
        let y = if m <= 2 { y - 1 } else { y };
        let era = if y >= 0 { y } else { y - 399 } / 400;
        let yoe = y - era * 400;
        let mp = i64::from((m + 9) % 12);
        let doy = (153 * mp + 2) / 5 + i64::from(d) - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }

    fn c(y: i64, m: u32, d: u32, hh: u32, mm: u32) -> Clock {
        Clock { y, m, d, hh, mm, days: days_from_civil(y, m, d) }
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
        assert!(!friendly_time(Some(clock(i64::MIN, i32::MIN)), now).is_empty());
        assert!(!friendly_time(Some(clock(i64::MAX, i32::MAX)), now).is_empty());
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
        let mut ui = crate::UiState { recent: vec!["/a".into(), "/b".into()], ..Default::default() };
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

    fn harness(a: crate::WordApp) -> egui_kittest::Harness<'static, crate::WordApp> {
        egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut crate::WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            a,
        )
    }

    fn app_with_recent(paths: &[&str]) -> crate::WordApp {
        let mut a = crate::WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), crate::Services::default());
        a.ui.recent = paths.iter().map(|p| p.to_string()).collect();
        a.run("ui.backstage", serde_json::json!({"value": true, "page": "home"})).unwrap();
        a
    }

    #[test]
    fn home_has_tabs_search_and_all_templates() {
        use egui_kittest::kittest::Queryable;
        let mut h = harness(app_with_recent(&[]));
        h.run_steps(4);
        for l in ["Recent", "Pinned", "Report", "Blank document", "Open file…", "New document"] {
            assert!(h.query_by_label(l).is_some(), "{l}");
        }
        assert!(h.query_by_role(egui::accesskit::Role::TextInput).is_some(), "search box");
    }

    #[test]
    fn pin_from_menu_moves_to_pinned_tab() {
        use egui_kittest::kittest::Queryable;
        let mut h = harness(app_with_recent(&["/nowhere/a.docx"]));
        h.run_steps(4);
        h.get_by_label("More actions for a.docx").click();
        h.run_steps(3);
        h.get_by_label("Pin").click();
        h.run_steps(3);
        assert_eq!(h.state().ui.pinned, vec!["/nowhere/a.docx"]);
        h.get_by_label("Pinned").click();
        h.run_steps(3);
        assert!(h.query_by_label("a.docx").is_some());
        assert!(h.query_by_label("Pin documents to keep them here.").is_none());
    }

    #[test]
    fn remove_from_list_drops_recent() {
        use egui_kittest::kittest::Queryable;
        let mut h = harness(app_with_recent(&["/nowhere/a.docx", "/nowhere/b.md"]));
        h.run_steps(4);
        h.get_by_label("More actions for a.docx").click();
        h.run_steps(3);
        h.get_by_label("Remove from list").click();
        h.run_steps(3);
        assert_eq!(h.state().ui.recent, vec!["/nowhere/b.md"]);
    }

    #[test]
    fn search_filters_documents_and_templates() {
        use egui_kittest::kittest::Queryable;
        let mut h = harness(app_with_recent(&["/nowhere/a.docx", "/nowhere/budget.md"]));
        h.run_steps(4);
        h.get_by_role(egui::accesskit::Role::TextInput).focus();
        h.run_steps(1);
        h.get_by_role(egui::accesskit::Role::TextInput).type_text("budget");
        h.run_steps(3);
        assert!(h.query_by_label("budget.md").is_some());
        assert!(h.query_by_label("a.docx").is_none());
        assert!(h.query_by_label("Letter").is_none());
    }

    #[test]
    fn missing_recent_row_is_dimmed_not_fatal() {
        use egui_kittest::kittest::Queryable;
        let mut h = harness(app_with_recent(&["/definitely/not/here.docx"]));
        h.run_steps(4);
        assert!(h.query_by_label("File not found").is_some());
    }

    #[test]
    fn show_in_finder_hidden_without_service() {
        use egui_kittest::kittest::Queryable;
        let mut h = harness(app_with_recent(&["/nowhere/a.docx"]));
        h.run_steps(4);
        h.get_by_label("More actions for a.docx").click();
        h.run_steps(3);
        assert!(h.query_by_label("Show in Finder").is_none() && h.query_by_label("Show in folder").is_none());
        assert!(h.query_by_label("Copy path").is_some());
    }
    #[test]
    fn dragging_a_file_over_home_shows_the_drop_hint() {
        use egui_kittest::kittest::Queryable;
        let mut h = harness(app_with_recent(&[]));
        h.run_steps(3);
        assert!(h.query_by_label("Drop to open").is_none());
        h.input_mut().hovered_files.push(egui::HoveredFile { path: Some("/x/a.docx".into()), ..Default::default() });
        h.run_steps(2);
        assert!(h.query_by_label("Drop to open").is_some());
    }
}
