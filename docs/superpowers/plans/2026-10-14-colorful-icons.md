# Colourful Icon Set Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Redraw WordCraft's 306 code-drawn icons as a colourful two-tone set (palette "colour by
function") without changing any icon name, call site or command.

**Architecture:** `icons.rs` gains a palette (`Family`, `Tones`), a name→family table, theme and
monochrome detection, and `Pen` helpers for rounded, filled, two-tone shapes. Task 1 also routes
every *legacy* icon's accent to its family colour, so the whole set turns colourful at once; Tasks
2–6 then redraw the icons tab by tab in the new style.

**Tech Stack:** Rust, egui 0.36 `Painter`/`Shape`, egui_kittest (wgpu) for the `icon_sheet` and
`ui_shot` examples.

**Spec:** `docs/superpowers/specs/2026-10-14-colorful-icons-design.md`

## Global Constraints

- `icons::paint(p, r, name, c, accent)` keeps its signature; no call site changes.
- Monochrome when `accent == c` or `accent.a() == 0`: every part in `c`.
- Data-colour icons keep `accent` for the chosen-colour part: `fontcolor`, `highlight`, `shading`, `textFill`, `textOutline`, `pageColor`, `pen`, `pencil`, `borderPainter`.
- Dark theme ⇔ luminance of `c` > 0.5.
- Stroke width 1.4 grid units, round caps/joins; rectangles radius 1.5–2.
- Palette hex values exactly as the spec table.
- No `unwrap/expect/panic!/[i]` in non-test code; original artwork only.
- Builds: `CARGO_TARGET_DIR=$TMPDIR/wordcraft-dev`, `PATH=/opt/homebrew/opt/rustup/bin:$PATH`; delete the dir when done; reinstall with `packaging/macos/install-local.sh`.
- Commit per task: `ICON.n: …`; `cargo xtask ci` green except the known upstream `turkish_proofing_through_the_review_commands`.

## Review Focus

- Disabled ribbon buttons (callers pass `c == a == text_disabled`): must stay fully grey. (Task 1 `monochrome_when_accent_equals_ink`.)
- Read Aloud passes a transparent accent when disabled: must be monochrome, not invisible parts. (Task 1 `monochrome_when_accent_transparent`.)
- Ink pen buttons pass the pen colour as accent: the pen tip keeps that colour. (Task 1 `data_colour_icons_keep_accent`.)
- Dark theme (`c` = `#DDDDDD`): palette switches to dark tones. (Task 1 `dark_ink_picks_dark_tones`.)
- An unknown icon name: still draws the letter tile, never panics. (Task 1 `unknown_name_draws_fallback`.)

---

### Task 1: Palette, families, monochrome, Pen helpers; chrome and generic icons

**Files:**
- Modify: `crates/ui-egui/src/icons.rs`

**Interfaces:**
- Produces (used by Tasks 2–6):
  - `pub enum Family { Neutral, Orange, Red, Blue, Green, Purple, Teal, Amber }`
  - `pub struct Tones { pub ink: Color32, pub tint: Color32, pub solid: Color32 }`
  - `pub fn tones(f: Family, dark: bool) -> Tones`
  - `pub fn family(name: &str) -> Family`
  - `Pen` fields `k: Tones` (icon family), `n: Tones` (neutral), `mono: bool`; methods
    `rr(x0, y0, x1, y1, radius, fill: Color32, stroke: Color32)` (rounded rect, `TRANSPARENT` skips a part),
    `poly(pts, fill, stroke)` (filled + outlined convex polygon; concave art is split into convex parts),
    `seg(pts, colour)` (polyline with round caps), `dot(x, y, r, colour)`.
  - In monochrome mode the tones are `ink = c`, `tint = c.gamma_multiply(0.22)`, `solid = c`, for both `k` and `n`.
  - Legacy art (anything still using `c`/`a`/`pen.a`) gets `a = k.solid` in colour mode (data-colour icons: `a = accent`).

- [ ] **Step 1: Write failing tests** (`#[cfg(test)] mod tests` at the bottom of `icons.rs`)

```rust
use super::*;

/// Paint `name` and return the unmultiplied RGB of every visible fill, stroke and text colour.
fn colours(name: &str, c: Color32, a: Color32) -> Vec<[u8; 3]> {
    let ctx = egui::Context::default();
    let _ = ctx.run(Default::default(), |ctx| crate::theme::install_fonts(ctx));
    let out = ctx.run(Default::default(), |ctx| {
        let p = ctx.layer_painter(egui::LayerId::background());
        paint(&p, Rect::from_min_size(pos2(0.0, 0.0), vec2(32.0, 32.0)), name, c, a);
    });
    let mut v = Vec::new();
    fn walk(s: &Shape, v: &mut Vec<[u8; 3]>) {
        let mut push = |c: Color32| {
            if c.a() > 0 {
                let [r, g, b, _] = c.to_srgba_unmultiplied();
                v.push([r, g, b]);
            }
        };
        match s {
            Shape::Vec(xs) => xs.iter().for_each(|x| walk(x, v)),
            Shape::Rect(r) => { push(r.fill); push(r.stroke.color); }
            Shape::Circle(c) => { push(c.fill); push(c.stroke.color); }
            Shape::Path(pth) => { push(pth.fill); if let egui::epaint::ColorMode::Solid(c) = pth.stroke.color { push(c); } }
            Shape::LineSegment { stroke, .. } => push(stroke.color),
            Shape::Text(t) => { push(t.fallback_color); for r in &t.galley.job.sections { push(r.format.color); } }
            _ => {}
        }
    }
    for cs in &out.shapes { walk(&cs.shape, &mut v); }
    v
}
fn near(a: [u8; 3], b: Color32) -> bool {
    let [r, g, bb, _] = b.to_srgba_unmultiplied();
    a.iter().zip([r, g, bb]).all(|(x, y)| x.abs_diff(y) <= 3)
}
const INK: Color32 = Color32::from_rgb(0x42, 0x42, 0x42);
const DARK_INK: Color32 = Color32::from_rgb(0xDD, 0xDD, 0xDD);
const ACCENT: Color32 = Color32::from_rgb(0x2B, 0x57, 0x9A);

#[test]
fn monochrome_when_accent_equals_ink() {
    let grey = Color32::from_rgb(0xA6, 0xA6, 0xA6);
    for name in ["paste", "save", "table", "picture", "comment", "bold"] {
        let cs = colours(name, grey, grey);
        assert!(!cs.is_empty(), "{name}");
        assert!(cs.iter().all(|c| near(*c, grey)), "{name}: {cs:?}");
    }
}
#[test]
fn monochrome_when_accent_transparent() {
    let cs = colours("replayPlay", INK, Color32::TRANSPARENT);
    assert!(!cs.is_empty() && cs.iter().all(|c| near(*c, INK)), "{cs:?}");
}
#[test]
fn colour_mode_uses_the_family_palette() {
    let cs = colours("paste", INK, ACCENT);
    let orange = tones(Family::Orange, false);
    assert!(cs.iter().any(|c| near(*c, orange.solid)), "{cs:?}");
    assert!(!cs.iter().any(|c| near(*c, ACCENT)), "accent is replaced: {cs:?}");
}
#[test]
fn dark_ink_picks_dark_tones() {
    let cs = colours("paste", DARK_INK, ACCENT);
    assert!(cs.iter().any(|c| near(*c, tones(Family::Orange, true).solid)), "{cs:?}");
}
#[test]
fn data_colour_icons_keep_accent() {
    let chosen = Color32::from_rgb(0x11, 0xAA, 0x33);
    for name in ["fontcolor", "highlight", "pen", "pencil"] {
        assert!(colours(name, INK, chosen).iter().any(|c| near(*c, chosen)), "{name}");
    }
}
#[test]
fn families_follow_the_spec_groups() {
    assert_eq!(family("paste"), Family::Orange);
    assert_eq!(family("bold"), Family::Red);
    assert_eq!(family("highlight"), Family::Amber);
    assert_eq!(family("bullets"), Family::Blue);
    assert_eq!(family("table"), Family::Green);
    assert_eq!(family("newComment"), Family::Purple);
    assert_eq!(family("accept"), Family::Green);
    assert_eq!(family("reject"), Family::Red);
    assert_eq!(family("margins"), Family::Teal);
    assert_eq!(family("envelope"), Family::Amber);
    assert_eq!(family("dropdown"), Family::Neutral);
    assert_eq!(family("save"), Family::Blue);
}
#[test]
fn unknown_name_draws_fallback() {
    assert!(!colours("no-such-icon", INK, ACCENT).is_empty());
}
```

- [ ] **Step 2: Run, expect compile failure**: `cargo test -p wordcraft-ui-egui --lib icons::tests` → FAIL (`Family`, `tones`, `family` missing).

- [ ] **Step 3: Implement palette and families**

```rust
/// Colour families of the icon set (spec: colour by function).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family { Neutral, Orange, Red, Blue, Green, Purple, Teal, Amber }

/// Linework, light fill and strong fill of a family in one theme.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tones { pub ink: Color32, pub tint: Color32, pub solid: Color32 }

const fn rgb(v: u32) -> Color32 { Color32::from_rgb((v >> 16) as u8, (v >> 8) as u8, v as u8) }

pub fn tones(f: Family, dark: bool) -> Tones {
    let (ink, tint, solid) = match (f, dark) {
        (Family::Neutral, false) => (0x424242, 0xFFFFFF, 0x6B6B6B),
        (Family::Neutral, true) => (0xDDDDDD, 0x3A3A3A, 0xB5B5B5),
        (Family::Orange, false) => (0x7A4A12, 0xFBE3C2, 0xE8891C),
        (Family::Orange, true) => (0xF6C48A, 0x5A3A14, 0xF29A35),
        (Family::Red, false) => (0x7A1F1F, 0xFBD5D5, 0xD93636),
        (Family::Red, true) => (0xF5A8A8, 0x5C2222, 0xF05A5A),
        (Family::Blue, false) => (0x1F3F73, 0xD6E6FB, 0x2F7BE0),
        (Family::Blue, true) => (0xB9D3F7, 0x1D3A5E, 0x5A9BF0),
        (Family::Green, false) => (0x1E6B3A, 0xD7F1DF, 0x2EA35A),
        (Family::Green, true) => (0x9FDDB4, 0x1D4A2C, 0x45C273),
        (Family::Purple, false) => (0x4B2F86, 0xE7DDFB, 0x7C4FD9),
        (Family::Purple, true) => (0xCDB8F5, 0x3A2766, 0x9B76EE),
        (Family::Teal, false) => (0x0F5C55, 0xD2F0EC, 0x1C9C8F),
        (Family::Teal, true) => (0x9ADED6, 0x16443F, 0x3CC2B3),
        (Family::Amber, false) => (0x6E5200, 0xFDEFC3, 0xF2B705),
        (Family::Amber, true) => (0xF5DA8A, 0x4D3D0E, 0xF5C842),
    };
    Tones { ink: rgb(ink), tint: rgb(tint), solid: rgb(solid) }
}

/// Icons whose accent part shows the user's chosen colour.
const DATA_COLOUR: [&str; 9] = ["fontcolor", "highlight", "shading", "textFill", "textOutline", "pageColor", "pen", "pencil", "borderPainter"];

pub fn family(name: &str) -> Family {
    use Family::*;
    match name {
        "paste" | "cut" | "copy" | "painter" | "pen" | "pencil" | "eraser" | "thickness" | "lasso" | "inkToShape" | "inkToMath" | "canvas"
        | "replay" | "addPen" | "shapes" | "shapeEffects" | "addShape" | "wordArt" | "textFill" | "textOutline" | "transform" | "draw" | "drawTable"
        | "borderPainter" | "hideInk" => Orange,
        "bold" | "italic" | "underline" | "strike" | "charborder" | "enclose" | "subscript" | "superscript" | "grow" | "shrink" | "case" | "clear"
        | "phonetic" | "fontcolor" | "effects" | "fontdialog" | "reject" | "deleteComment" | "deleteTable" | "checkErrors" | "closeOutline" => Red,
        "highlight" | "envelope" | "labels" | "mailMerge" | "recipients" | "editRecipients" | "mergeField" | "addressBlock" | "greetingLine"
        | "rules" | "matchFields" | "highlightFields" | "preview" | "first" | "previous" | "next" | "last" | "finish" | "folder" | "homeFolder" => Amber,
        "coverPage" | "blankPage" | "pageBreak" | "table" | "picture" | "onlinePicture" | "icons" | "models3d" | "smartArt" | "chart" | "screenshot"
        | "video" | "link" | "bookmark" | "crossRef" | "header" | "footer" | "pageNumber" | "textBox" | "quickParts" | "dropCap" | "signature"
        | "dateTime" | "object" | "equation" | "symbol" | "accept" | "insertAbove" | "insertBelow" | "insertLeft" | "insertRight" | "merge"
        | "splitCells" | "splitTable" | "autofit" | "distributeRows" | "distributeCols" | "rowHeight" | "colWidth" | "cellAlign"
        | "textDirection" | "cellMargins" | "repeatHeader" | "toText" | "formula" | "selectTable" | "promote" | "demote" | "textPane"
        | "smartArtColors" | "resetGraphic" | "changeChartType" | "chartData" | "chartTitle" | "legend" | "dataLabels" | "addins" | "setDefault"
        | "ccRichText" | "ccPlainText" | "ccPicture" | "ccGallery" | "ccCheckBox" | "ccComboBox" | "ccDropDown" | "ccDate" | "ccRepeat"
        | "designMode" | "plusCircle" => Green,
        "newComment" | "comment" | "prevComment" | "nextComment" | "showComments" | "resolve" | "trackChanges" | "markup" | "reviewingPane"
        | "prevChange" | "nextChange" | "compare" | "combine" | "protect" | "restrict" | "blockAuthors" | "spelling" | "editor" | "thesaurus"
        | "wordCount" | "accessibility" | "altText" | "translate" | "language" | "styles" | "stylesPane" | "themes" | "colors" | "fonts"
        | "paraSpacing" | "effectsDesign" | "watermark" | "pageColor" | "pageBorders" => Purple,
        "margins" | "orientation" | "size" | "columns" | "breaks" | "lineNumbers" | "hyphenation" | "position" | "wrapText" | "bringForward"
        | "sendBackward" | "selectionPane" | "align" | "group" | "rotate" | "indentLeft" | "indentRight" | "spaceBefore" | "spaceAfter" | "toc"
        | "addText" | "updateTable" | "update" | "footnote" | "endnote" | "nextFootnote" | "showNotes" | "researcher" | "docPrefs"
        | "unlinkCitations" | "addNote" | "citation" | "sources" | "bibliography" | "caption" | "tableOfFigures" | "markEntry" | "index"
        | "markCitation" | "tableOfAuthorities" | "pastCitation" | "insertCitation" => Teal,
        "dropdown" | "more" | "launcher" | "plus" | "minus" | "check" | "close" | "chevronLeft" | "chevronRight" | "chevronUp" | "chevronDown"
        | "chevronDoubleRight" | "help" | "discord" => Neutral,
        _ => Blue,
    }
}
```

Names not listed (paragraph, editing, chrome, view, media, outline…) are Blue; an unknown name is Blue too, which only tints its letter-tile fallback.

- [ ] **Step 4: Monochrome + Pen.** Split `paint` into `paint` (palette, mono, fallback) and `fn art(pen: &Pen, name: &str) -> bool` (the existing big match, returning `true`, and `false` for `_`). In `paint`:

```rust
pub fn paint(p: &Painter, r: Rect, name: &str, c: Color32, accent: Color32) {
    let mono = accent == c || accent.a() == 0;
    let [cr, cg, cb, _] = c.to_srgba_unmultiplied();
    let dark = 0.2126 * f32::from(cr) + 0.7152 * f32::from(cg) + 0.0722 * f32::from(cb) > 127.0;
    let (k, n) = if mono {
        let t = Tones { ink: c, tint: c.gamma_multiply(0.22), solid: c };
        (t, t)
    } else {
        (tones(family(name), dark), tones(Family::Neutral, dark))
    };
    let a = if mono { c } else if DATA_COLOUR.contains(&name) { accent } else { k.solid };
    let pen = Pen { p, r, c, a, w: 1.4, k, n, mono };
    if !art(&pen, name) {
        fallback(p, r, name, c, if mono { c } else { k.solid });
    }
}
```

Legacy art that hard-codes `red/yellow/green/orange` locals: in mono those must become `c`: define them in `art` as `let red = if pen.mono { pen.c } else { rgb(0xD13B3B) };` etc. Add the Pen helpers listed in Interfaces (stroke uses `Stroke::new(self.w * self.s(), colour)`; `rr` uses `rect_filled` + `rect_stroke` with `CornerRadius::same((radius * s) as u8)` clamped `0..=255`).

Then redraw the chrome/generic set in the new style (all use `pen.k`/`pen.n`): `save` (solid blue body, white label and shutter), `undo`/`redo` (ink curve, solid arrowhead), `search`/`find` (tint lens, ink rim, solid handle), `share` (tint tray, solid arrow), `user` (solid head/body), `open`/`folder` (amber tint back, amber solid front), `file`, `print` (neutral body, blue paper slot), `info`/`properties` (solid circle, white i), `help` (neutral circle, ?), `pin` (solid head), `close`, `plus`, `minus`, `check`, `more`, `dropdown`, `launcher`, `chevron*` (neutral ink, round caps), `homeFolder` (amber roof, tint walls), `docPrefs` (page + teal gear), `discord` (keep neutral), `darkMode`, `interfaceTheme`.

- [ ] **Step 5: Run tests, expect PASS**: `cargo test -p wordcraft-ui-egui --lib icons::tests`; then `cargo test -p wordcraft-ui-egui` (whole crate).
- [ ] **Step 6: Look.** `cargo run -p wordcraft-ui-egui --example icon_sheet -- $S/icons_t1.png` and `… dark`; `ui_shot` of the document view (`{"shot": …}`) and Home. Read the PNGs; fix contrast or clutter.
- [ ] **Step 7: Commit** `ICON.1: icon palette, families, monochrome mode; chrome icons redrawn`

### Tasks 2–6: Redraw batches

Each batch: redraw every listed icon with `rr/poly/seg/dot` using `pen.k` (family) and `pen.n`
(paper/neutral), following the art rules below; run `cargo test -p wordcraft-ui-egui --lib icons::tests`
(must stay green — monochrome tests cover every redrawn icon automatically once added to the
`monochrome_when_accent_equals_ink` list: append the batch's names to that list in the same commit);
render `icon_sheet` light+dark and a `ui_shot` of the batch's ribbon tab (`{"method":"engine.execute","params":{"command":"ui.tab","params":{"value":"<tab>"}}}` — check the command id with `wordcraft-cli commands | grep ui.tab` first); fix what looks wrong; commit.

Art rules (all batches):
- Paper/pages: `n.tint` fill (white in light), `n.ink` outline, folded corner as a small `n.ink` line.
- The icon's subject: `k.solid` fill; secondary parts `k.tint` fill with `k.ink` outline.
- Text lines inside pages: `k.ink` at 1.2 width, round caps; never more than 3 lines at 16 pt.
- At most two colour families per icon (family + neutral; amber allowed for a "sun/star/highlight" detail).
- Letter glyphs (B, I, U, Aa, abc): `k.ink` text; the decorating bar/underline in `k.solid`.
- Keep silhouettes recognisable from the current art (same metaphor, same placement of arrows), so users don't relearn.

- **Task 2 — Home tab** (`ICON.2`): paste cut copy painter bold italic underline strike charborder enclose subscript superscript grow shrink case clear phonetic highlight fontcolor effects fontdialog bullets numbering multilevel outdent indent sort pilcrow alignLeft alignCenter alignRight justify textLtr textRtl lineSpacing shading borders styles stylesPane find replace select dictate editor.
- **Task 3 — Insert tab** (`ICON.3`): coverPage blankPage pageBreak table drawTable picture onlinePicture shapes shapeEffects icons models3d smartArt addShape promote demote textPane smartArtColors resetGraphic chart changeChartType chartData chartTitle legend dataLabels screenshot video link bookmark crossRef newComment comment header footer pageNumber textBox quickParts wordArt textFill textOutline transform dropCap signature dateTime object equation symbol addins.
- **Task 4 — Draw, Design, Layout** (`ICON.4`): pen pencil eraser thickness lasso inkToShape inkToMath canvas replay addPen replayRewind replayPlay replayPause replayForward themes colors fonts paraSpacing effectsDesign setDefault watermark pageColor pageBorders margins orientation size columns breaks lineNumbers hyphenation position wrapText bringForward sendBackward selectionPane align group rotate indentLeft indentRight spaceBefore spaceAfter.
- **Task 5 — References, Mailings** (`ICON.5`): toc addText updateTable update footnote endnote nextFootnote showNotes researcher docPrefs unlinkCitations addNote citation sources bibliography caption tableOfFigures markEntry index markCitation tableOfAuthorities pastCitation envelope labels mailMerge recipients editRecipients mergeField addressBlock greetingLine rules matchFields highlightFields preview next previous first last finish checkErrors.
- **Task 6 — Review, View, table tabs, content controls, media, rest** (`ICON.6`): spelling thesaurus wordCount readAloud accessibility altText translate language deleteComment prevComment nextComment showComments resolve trackChanges markup reviewingPane accept reject prevChange nextChange compare combine protect restrict blockAuthors hideInk readMode printLayout webLayout outline draft focus immersive vertical sideToSide ruler gridlines navPane zoom zoomIn zoomOut zoom100 onePage multiplePages pageWidth newWindow arrangeAll split fullScreen sideBySide syncScroll switchWindows macros documentTemplate insertAbove insertBelow insertLeft insertRight deleteTable merge splitCells splitTable autofit distributeRows distributeCols rowHeight colWidth cellAlign textDirection cellMargins repeatHeader toText formula selectTable draw borderPainter media* outline* cc* designMode alignText textBoxLink textBoxUnlink chevronDoubleRight, and any name the sheet still shows in the old style.

### Task 7: Finish

- [ ] Docs: `docs/ui-parity.md` revision row + status line; `ROADMAP.md` progress log; `ATTRIBUTION.md` needs no row (code-drawn, no asset files) — verify with `cargo xtask assets`.
- [ ] `cargo xtask ci`; merge to `main`, push to `origin`; reinstall the app; delete `$TMPDIR/wordcraft-dev`.
