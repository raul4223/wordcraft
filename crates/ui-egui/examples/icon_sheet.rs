//! Contact sheet of every code-drawn icon, for reviewing the icon set by eye.
//!
//! `cargo run -p wordcraft-ui-egui --example icon_sheet -- /abs/out.png [dark]`
//!
//! Icon names are read from `icons.rs` itself (the `"name" =>` match arms), so new icons show up
//! without editing this file. Each cell shows the icon at ribbon sizes 16 and 32 pt and its name.

use egui::{Color32, FontId, Rect, pos2, vec2};

const COLS: usize = 12;
const CELL: egui::Vec2 = egui::Vec2 { x: 118.0, y: 74.0 };

fn names() -> Vec<String> {
    let src = include_str!("../src/icons.rs");
    let mut out = Vec::new();
    for line in src.lines() {
        let l = line.trim_start();
        if !l.starts_with('"') {
            continue;
        }
        let Some((arms, _)) = l.split_once("=>") else { continue };
        for arm in arms.split('|') {
            let n = arm.trim().trim_matches('"');
            if !n.is_empty() && n.chars().all(|c| c.is_ascii_alphanumeric()) && !out.iter().any(|x| x == n) {
                out.push(n.to_string());
            }
        }
    }
    out
}

fn main() {
    let path = std::env::args().nth(1).expect("usage: icon_sheet out.png [dark]");
    let dark = std::env::args().nth(2).is_some_and(|a| a == "dark");
    let names = names();
    let count = names.len();
    let rows = names.len().div_ceil(COLS);
    let size = vec2(COLS as f32 * CELL.x + 20.0, rows as f32 * CELL.y + 20.0);
    let mut fonts_ready = false;
    let mut h = egui_kittest::Harness::builder().with_size(size).with_pixels_per_point(2.0).wgpu().build_ui(move |ui| {
        if !fonts_ready {
            wordcraft_ui_egui::theme::install_fonts(ui.ctx());
            fonts_ready = true;
            return;
        }
        let (bg, ink, dim) = if dark {
            (Color32::from_rgb(0x29, 0x29, 0x29), Color32::from_rgb(0xDD, 0xDD, 0xDD), Color32::from_gray(150))
        } else {
            (Color32::from_rgb(0xF7, 0xF7, 0xF7), Color32::from_rgb(0x42, 0x42, 0x42), Color32::from_gray(120))
        };
        let accent = Color32::from_rgb(0x2B, 0x57, 0x9A);
        let p = ui.painter();
        p.rect_filled(ui.max_rect(), 0.0, bg);
        for (i, n) in names.iter().enumerate() {
            let o = pos2(10.0 + (i % COLS) as f32 * CELL.x, 10.0 + (i / COLS) as f32 * CELL.y);
            wordcraft_ui_egui::icons::paint(p, Rect::from_min_size(o + vec2(8.0, 14.0), vec2(16.0, 16.0)), n, ink, accent);
            wordcraft_ui_egui::icons::paint(p, Rect::from_min_size(o + vec2(36.0, 6.0), vec2(32.0, 32.0)), n, ink, accent);
            p.text(o + vec2(4.0, 50.0), egui::Align2::LEFT_TOP, n, FontId::proportional(10.0), dim);
        }
    });
    h.run_steps(4);
    match h.render() {
        Ok(img) => {
            img.save(&path).expect("save png");
            println!("{path} ({count} icons)");
        }
        Err(e) => eprintln!("render failed: {e}"),
    }
}
