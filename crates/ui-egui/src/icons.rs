//! WordCraft's icon set, drawn in code (original artwork; no external icon assets).
//!
//! Icons are designed on a 20×20 grid in two colours: line work (`c`) and an accent (`a`).

use egui::{Color32, FontId, Painter, Pos2, Rect, Shape, Stroke, pos2, vec2};

struct Pen<'a> {
    p: &'a Painter,
    r: Rect,
    c: Color32,
    a: Color32,
    w: f32,
    /// The icon's family tones and the neutral ones (both `c` in monochrome mode).
    k: Tones,
    n: Tones,
    mono: bool,
}

impl<'a> Pen<'a> {
    fn s(&self) -> f32 {
        self.r.width().min(self.r.height()) / 20.0
    }
    fn pt(&self, x: f32, y: f32) -> Pos2 {
        let s = self.s();
        let o = self.r.center() - vec2(10.0 * s, 10.0 * s);
        pos2(o.x + x * s, o.y + y * s)
    }
    fn stroke(&self, c: Color32) -> Stroke {
        Stroke::new(self.w * self.s(), c)
    }
    fn line(&self, pts: &[(f32, f32)]) {
        self.line_c(pts, self.c);
    }
    fn line_c(&self, pts: &[(f32, f32)], c: Color32) {
        let v: Vec<Pos2> = pts.iter().map(|&(x, y)| self.pt(x, y)).collect();
        self.p.add(Shape::line(v, self.stroke(c)));
    }
    fn closed(&self, pts: &[(f32, f32)], c: Color32) {
        let v: Vec<Pos2> = pts.iter().map(|&(x, y)| self.pt(x, y)).collect();
        self.p.add(Shape::closed_line(v, self.stroke(c)));
    }
    fn fill(&self, pts: &[(f32, f32)], c: Color32) {
        let v: Vec<Pos2> = pts.iter().map(|&(x, y)| self.pt(x, y)).collect();
        self.p.add(Shape::convex_polygon(v, c, Stroke::NONE));
    }
    fn rect(&self, x0: f32, y0: f32, x1: f32, y1: f32, c: Color32) {
        self.p.rect_stroke(Rect::from_min_max(self.pt(x0, y0), self.pt(x1, y1)), 1.0, self.stroke(c), egui::StrokeKind::Middle);
    }
    fn frect(&self, x0: f32, y0: f32, x1: f32, y1: f32, c: Color32) {
        self.p.rect_filled(Rect::from_min_max(self.pt(x0, y0), self.pt(x1, y1)), 0.5, c);
    }
    fn circle(&self, x: f32, y: f32, r: f32, c: Color32) {
        self.p.circle_stroke(self.pt(x, y), r * self.s(), self.stroke(c));
    }
    fn fcircle(&self, x: f32, y: f32, r: f32, c: Color32) {
        self.p.circle_filled(self.pt(x, y), r * self.s(), c);
    }
    fn text(&self, x: f32, y: f32, size: f32, t: &str, c: Color32, bold: bool) {
        let font = if bold { crate::theme::semibold(size * self.s()) } else { FontId::proportional(size * self.s()) };
        self.p.text(self.pt(x, y), egui::Align2::CENTER_CENTER, t, font, c);
    }
    /// Horizontal text lines (a paragraph).
    fn lines(&self, x0: f32, x1: f32, ys: &[f32]) {
        for y in ys {
            self.line(&[(x0, *y), (x1, *y)]);
        }
    }
    /// A page outline with a folded corner.
    fn page(&self, x0: f32, y0: f32, x1: f32, y1: f32) {
        self.closed(&[(x0, y0), (x1 - 4.0, y0), (x1, y0 + 4.0), (x1, y1), (x0, y1)], self.c);
        self.line(&[(x1 - 4.0, y0), (x1 - 4.0, y0 + 4.0), (x1, y0 + 4.0)]);
    }
    /// Rounded rectangle; a `TRANSPARENT` fill or stroke is left out.
    #[allow(clippy::too_many_arguments)]
    fn rr(&self, x0: f32, y0: f32, x1: f32, y1: f32, radius: f32, fill: Color32, stroke: Color32) {
        let rect = Rect::from_min_max(self.pt(x0, y0), self.pt(x1, y1));
        let cr = egui::CornerRadius::same((radius * self.s()).clamp(0.0, 255.0) as u8);
        if fill.a() > 0 {
            self.p.rect_filled(rect, cr, fill);
        }
        if stroke.a() > 0 {
            self.p.rect_stroke(rect, cr, self.stroke(stroke), egui::StrokeKind::Middle);
        }
    }
    /// A convex polygon, filled and/or outlined (`TRANSPARENT` leaves a part out).
    fn poly(&self, pts: &[(f32, f32)], fill: Color32, stroke: Color32) {
        let v: Vec<Pos2> = pts.iter().map(|&(x, y)| self.pt(x, y)).collect();
        let st = if stroke.a() > 0 { self.stroke(stroke) } else { Stroke::NONE };
        self.p.add(Shape::convex_polygon(v, fill, st));
    }
    /// A polyline with round caps and joins.
    fn seg(&self, pts: &[(f32, f32)], c: Color32) {
        let v: Vec<Pos2> = pts.iter().map(|&(x, y)| self.pt(x, y)).collect();
        let half = self.w * self.s() / 2.0;
        for q in &v {
            self.p.circle_filled(*q, half, c);
        }
        self.p.add(Shape::line(v, self.stroke(c)));
    }
    /// The same pen drawing into another box (an icon inside an icon).
    fn at(&self, r: Rect) -> Pen<'a> {
        Pen { p: self.p, r, c: self.c, a: self.a, w: self.w, k: self.k, n: self.n, mono: self.mono }
    }
    /// A polyline `k` times as heavy as the pen, with round caps.
    fn heavy(&self, pts: &[(f32, f32)], c: Color32, k: f32) {
        let v: Vec<Pos2> = pts.iter().map(|&(x, y)| self.pt(x, y)).collect();
        let w = self.w * k * self.s();
        for q in &v {
            self.p.circle_filled(*q, w / 2.0, c);
        }
        self.p.add(Shape::line(v, Stroke::new(w, c)));
    }
    /// A filled and outlined circle (`TRANSPARENT` leaves a part out).
    fn disc(&self, x: f32, y: f32, r: f32, fill: Color32, stroke: Color32) {
        if fill.a() > 0 {
            self.p.circle_filled(self.pt(x, y), r * self.s(), fill);
        }
        if stroke.a() > 0 {
            self.p.circle_stroke(self.pt(x, y), r * self.s(), self.stroke(stroke));
        }
    }
    /// Badge colours: a solid disc with a light glyph, or (one colour) a tinted disc with an ink glyph.
    fn badge(&self) -> (Color32, Color32) {
        if self.mono { (self.k.tint, self.k.ink) } else { (self.k.solid, self.n.tint) }
    }
    /// A page with a folded corner: paper fill, neutral outline.
    #[allow(dead_code)] // first used by the Insert tab batch
    fn sheet(&self, x0: f32, y0: f32, x1: f32, y1: f32) {
        let n = self.n;
        self.poly(&[(x0, y0), (x1 - 4.0, y0), (x1, y0 + 4.0), (x1, y1), (x0, y1)], n.tint, n.ink);
        self.poly(&[(x1 - 4.0, y0), (x1, y0 + 4.0), (x1 - 4.0, y0 + 4.0)], self.k.tint, n.ink);
    }
    /// Horizontal text lines with round caps.
    fn rows(&self, x0: f32, x1: f32, ys: &[f32], c: Color32) {
        for y in ys {
            self.seg(&[(x0, *y), (x1, *y)], c);
        }
    }
    /// A filled dot.
    fn dot(&self, x: f32, y: f32, r: f32, c: Color32) {
        self.p.circle_filled(self.pt(x, y), r * self.s(), c);
    }
    fn arrow_down(&self, x: f32, y: f32, c: Color32) {
        self.fill(&[(x - 2.5, y - 1.2), (x + 2.5, y - 1.2), (x, y + 1.5)], c);
    }
}

/// Colour families of the icon set (spec: colour by function).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    Neutral,
    Orange,
    Red,
    Blue,
    Green,
    Purple,
    Teal,
    Amber,
}

/// Linework, light fill and strong fill of a family in one theme.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tones {
    pub ink: Color32,
    pub tint: Color32,
    pub solid: Color32,
}

const fn rgb(v: u32) -> Color32 {
    Color32::from_rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

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

/// Icons whose accent part shows the user's chosen colour (the Draw tab's pen buttons pass the
/// pen's colour). Font colour and highlight buttons paint the chosen colour over the icon instead.
const DATA_COLOUR: [&str; 2] = ["pen", "pencil"];

pub fn family(name: &str) -> Family {
    use Family::*;
    match name {
        "paste" | "cut" | "copy" | "painter" | "pen" | "pencil" | "eraser" | "thickness" | "lasso" | "inkToShape" | "inkToMath" | "canvas"
        | "replay" | "addPen" | "shapes" | "shapeEffects" | "addShape" | "wordArt" | "textFill" | "textOutline" | "transform" | "draw"
        | "drawTable" | "borderPainter" | "hideInk" => Orange,
        "bold" | "italic" | "underline" | "strike" | "charborder" | "enclose" | "subscript" | "superscript" | "grow" | "shrink" | "case"
        | "clear" | "phonetic" | "fontcolor" | "effects" | "fontdialog" | "reject" | "deleteComment" | "deleteTable" | "checkErrors"
        | "closeOutline" => Red,
        "highlight" | "envelope" | "labels" | "mailMerge" | "recipients" | "editRecipients" | "mergeField" | "addressBlock" | "greetingLine"
        | "rules" | "matchFields" | "highlightFields" | "preview" | "first" | "previous" | "next" | "last" | "finish" | "folder" | "homeFolder"
        | "open" => Amber,
        "coverPage" | "blankPage" | "pageBreak" | "table" | "picture" | "onlinePicture" | "icons" | "models3d" | "smartArt" | "chart"
        | "screenshot" | "video" | "link" | "bookmark" | "crossRef" | "header" | "footer" | "pageNumber" | "textBox" | "quickParts" | "dropCap"
        | "signature" | "dateTime" | "object" | "equation" | "symbol" | "accept" | "insertAbove" | "insertBelow" | "insertLeft" | "insertRight"
        | "merge" | "splitCells" | "splitTable" | "autofit" | "distributeRows" | "distributeCols" | "rowHeight" | "colWidth" | "cellAlign"
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

/// Paint icon `name` into `r`. `c` is the ink (and tells the theme: light ink means a dark theme).
/// `accent == c` or a transparent accent asks for a one-colour icon in `c` (disabled buttons,
/// icons on coloured bars); otherwise the icon's family palette colours it, and `accent` is only
/// used by icons that show a chosen colour (font colour, highlighter, pens…).
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
    let a = if mono {
        c
    } else if DATA_COLOUR.contains(&name) {
        accent
    } else {
        k.solid
    };
    let pen = Pen { p, r, c, a, w: 1.4, k, n, mono };
    if !art(&pen, name) {
        // Fallback: a rounded tile with the first letter.
        p.rect_stroke(r.shrink(r.width() * 0.12), 3.0, Stroke::new(1.0, c), egui::StrokeKind::Middle);
        let ch = name.chars().next().map(|x| x.to_ascii_uppercase().to_string()).unwrap_or_default();
        p.text(r.center(), egui::Align2::CENTER_CENTER, ch, FontId::proportional(r.height() * 0.45), a);
    }
}

/// Draw the art for `name`; false when there is none.
fn art(pen: &Pen, name: &str) -> bool {
    let (r, c, a) = (pen.r, pen.c, pen.a);
    let pick = |x: u32| if pen.mono { c } else { rgb(x) };
    let red = pick(0xD13B3B);
    let yellow = pick(0xF2C811);
    let green = pick(0x2E8B57);
    let orange = pick(0xE07B1F);
    match name {
        // Clipboard
        "paste" => {
            let (k, n) = (pen.k, pen.n);
            pen.rr(3.5, 3.5, 14.0, 17.5, 1.6, k.tint, k.ink);
            pen.rr(6.5, 1.8, 11.0, 5.2, 1.0, k.solid, k.ink);
            pen.rr(8.5, 8.0, 17.5, 18.5, 1.4, n.tint, k.ink);
            pen.rows(10.6, 15.4, &[11.2, 13.6, 16.0], k.solid);
        }
        "cut" => {
            let (k, n) = (pen.k, pen.n);
            pen.seg(&[(7.6, 12.6), (14.0, 2.8)], n.ink);
            pen.seg(&[(12.4, 12.6), (6.0, 2.8)], n.ink);
            pen.disc(6.0, 15.2, 2.9, k.solid, Color32::TRANSPARENT);
            pen.disc(14.0, 15.2, 2.9, k.solid, Color32::TRANSPARENT);
            pen.disc(6.0, 15.2, 1.1, n.tint, Color32::TRANSPARENT);
            pen.disc(14.0, 15.2, 1.1, n.tint, Color32::TRANSPARENT);
        }
        "copy" => {
            let (k, n) = (pen.k, pen.n);
            pen.rr(3.0, 2.5, 12.5, 13.5, 1.5, k.tint, k.ink);
            pen.rr(7.5, 6.5, 17.0, 17.5, 1.5, n.tint, k.ink);
            pen.rows(9.8, 14.6, &[10.2, 12.6, 15.0], k.solid);
        }
        "painter" => {
            let (k, n) = (pen.k, pen.n);
            pen.rr(3.5, 2.5, 15.5, 8.0, 1.5, k.solid, k.ink);
            pen.seg(&[(15.5, 5.2), (17.5, 5.2), (17.5, 10.0), (10.0, 10.0), (10.0, 12.0)], n.ink);
            pen.rr(8.6, 12.0, 11.4, 18.5, 1.0, k.tint, k.ink);
        }
        // Font
        "bold" => pen.text(10.0, 10.5, 15.5, "B", pen.k.ink, true),
        "italic" => pen.text(10.0, 10.5, 15.5, "I", pen.k.ink, false),
        "underline" => {
            pen.text(10.0, 8.8, 13.5, "U", pen.k.ink, false);
            pen.heavy(&[(5.0, 17.2), (15.0, 17.2)], pen.k.solid, 1.2);
        }
        "strike" => {
            pen.text(10.0, 10.0, 12.0, "ab", pen.k.ink, false);
            pen.heavy(&[(3.0, 10.6), (17.0, 10.6)], pen.k.solid, 1.1);
        }
        "charborder" => {
            pen.rr(3.0, 3.0, 17.0, 17.0, 2.0, pen.k.tint, pen.k.solid);
            pen.text(10.0, 10.0, 10.0, "A", pen.k.ink, true);
        }
        "enclose" => {
            pen.disc(10.0, 10.2, 7.5, pen.k.tint, pen.k.solid);
            pen.text(10.0, 10.2, 9.5, "A", pen.k.ink, true);
        }
        "subscript" => {
            pen.text(8.0, 9.0, 13.0, "x", pen.k.ink, false);
            pen.text(15.0, 14.5, 8.5, "2", pen.k.solid, true);
        }
        "superscript" => {
            pen.text(8.0, 11.0, 13.0, "x", pen.k.ink, false);
            pen.text(15.0, 5.0, 8.5, "2", pen.k.solid, true);
        }
        "grow" => {
            pen.text(8.0, 11.0, 15.0, "A", pen.k.ink, false);
            pen.poly(&[(14.0, 6.5), (19.0, 6.5), (16.5, 2.5)], pen.k.solid, Color32::TRANSPARENT);
        }
        "shrink" => {
            pen.text(8.0, 12.0, 11.0, "A", pen.k.ink, false);
            pen.poly(&[(14.0, 3.5), (19.0, 3.5), (16.5, 7.5)], pen.k.solid, Color32::TRANSPARENT);
        }
        "case" => {
            pen.text(7.0, 10.5, 13.0, "A", pen.k.ink, false);
            pen.text(14.0, 11.5, 10.0, "a", pen.k.solid, true);
        }
        "clear" => {
            pen.text(8.0, 9.0, 13.0, "A", pen.k.ink, false);
            let pink = if pen.mono { c } else { rgb(0xE8_6F_A0) };
            pen.poly(&[(10.0, 15.0), (14.0, 11.0), (18.0, 15.0), (14.0, 19.0)], pink, Color32::TRANSPARENT);
            pen.poly(&[(10.0, 15.0), (12.0, 13.0), (16.0, 17.0), (14.0, 19.0)], pen.n.tint, Color32::TRANSPARENT);
        }
        // Small letters over a large one: text with a reading above it.
        "phonetic" => {
            pen.text(10.0, 4.8, 7.5, "abc", pen.k.solid, true);
            pen.text(10.0, 13.5, 12.5, "A", pen.k.ink, false);
        }
        "highlight" => {
            let (k, n) = (pen.k, pen.n);
            pen.poly(&[(5.0, 12.0), (11.0, 4.0), (14.0, 7.0), (8.0, 15.0)], n.tint, n.ink);
            pen.poly(&[(11.0, 4.0), (12.5, 2.5), (15.5, 5.5), (14.0, 7.0)], n.ink, Color32::TRANSPARENT);
            pen.poly(&[(5.0, 12.0), (8.0, 15.0), (3.5, 15.5)], k.solid, Color32::TRANSPARENT);
            pen.rr(2.0, 16.5, 18.0, 19.2, 0.8, k.solid, Color32::TRANSPARENT);
        }
        "fontcolor" => {
            pen.text(10.0, 8.8, 13.5, "A", pen.n.ink, false);
            pen.rr(2.0, 16.5, 18.0, 19.2, 0.8, pen.k.solid, Color32::TRANSPARENT);
        }
        "effects" => {
            pen.text(10.6, 10.6, 15.0, "A", pen.k.tint, true);
            pen.text(10.0, 10.0, 15.0, "A", pen.k.solid, true);
        }
        "fontdialog" => pen.seg(&[(13.0, 13.0), (17.0, 17.0), (17.0, 13.0)], pen.n.ink),
        // Paragraph
        "bullets" => {
            for y in [5.0, 10.0, 15.0] {
                pen.dot(4.0, y, 1.7, pen.k.solid);
                pen.seg(&[(8.0, y), (17.0, y)], pen.k.ink);
            }
        }
        "numbering" => {
            for (i, y) in [5.0, 10.0, 15.0].iter().enumerate() {
                pen.text(4.0, *y, 6.5, &(i + 1).to_string(), pen.k.solid, true);
                pen.seg(&[(8.0, *y), (17.0, *y)], pen.k.ink);
            }
        }
        "multilevel" => {
            let k = pen.k;
            pen.text(3.5, 4.5, 6.0, "1", k.solid, true);
            pen.seg(&[(6.8, 4.5), (17.0, 4.5)], k.ink);
            pen.text(6.5, 10.0, 6.0, "a", k.solid, true);
            pen.seg(&[(9.8, 10.0), (17.0, 10.0)], k.ink);
            pen.text(9.5, 15.5, 6.0, "i", k.solid, true);
            pen.seg(&[(12.3, 15.5), (17.0, 15.5)], k.ink);
        }
        "outdent" | "indent" => {
            let k = pen.k;
            pen.rows(9.5, 17.0, &[7.0, 10.0, 13.0], k.ink);
            pen.rows(3.0, 17.0, &[3.5, 16.5], k.ink);
            if name == "indent" {
                pen.poly(&[(3.0, 6.8), (3.0, 13.2), (7.0, 10.0)], k.solid, Color32::TRANSPARENT);
            } else {
                pen.poly(&[(7.0, 6.8), (7.0, 13.2), (3.0, 10.0)], k.solid, Color32::TRANSPARENT);
            }
        }
        "sort" => {
            let k = pen.k;
            pen.text(6.0, 5.5, 7.5, "A", k.ink, true);
            pen.text(6.0, 14.5, 7.5, "Z", k.ink, true);
            pen.heavy(&[(14.0, 3.0), (14.0, 15.0)], k.solid, 1.1);
            pen.poly(&[(11.0, 13.5), (17.0, 13.5), (14.0, 18.0)], k.solid, Color32::TRANSPARENT);
        }
        "pilcrow" => pen.text(10.0, 10.5, 15.0, "¶", pen.k.solid, false),
        "alignLeft" => {
            pen.rows(3.0, 17.0, &[4.0, 10.0, 16.0], pen.k.ink);
            pen.rows(3.0, 12.0, &[7.0, 13.0], pen.k.solid);
        }
        "alignCenter" => {
            pen.rows(3.0, 17.0, &[4.0, 10.0, 16.0], pen.k.ink);
            pen.rows(6.0, 14.0, &[7.0, 13.0], pen.k.solid);
        }
        "alignRight" => {
            pen.rows(3.0, 17.0, &[4.0, 10.0, 16.0], pen.k.ink);
            pen.rows(8.0, 17.0, &[7.0, 13.0], pen.k.solid);
        }
        "justify" => {
            pen.rows(3.0, 17.0, &[4.0, 10.0, 16.0], pen.k.ink);
            pen.rows(3.0, 17.0, &[7.0, 13.0], pen.k.solid);
        }
        // Text direction: lines set from the start edge, an arrow in the reading direction.
        "textLtr" => {
            let k = pen.k;
            pen.rows(3.0, 14.0, &[4.0, 8.0], k.ink);
            pen.rows(3.0, 10.0, &[12.0], k.ink);
            pen.heavy(&[(3.0, 16.5), (14.0, 16.5)], k.solid, 1.1);
            pen.poly(&[(13.0, 13.8), (17.5, 16.5), (13.0, 19.2)], k.solid, Color32::TRANSPARENT);
        }
        "textRtl" => {
            let k = pen.k;
            pen.rows(6.0, 17.0, &[4.0, 8.0], k.ink);
            pen.rows(10.0, 17.0, &[12.0], k.ink);
            pen.heavy(&[(17.0, 16.5), (6.0, 16.5)], k.solid, 1.1);
            pen.poly(&[(7.0, 13.8), (2.5, 16.5), (7.0, 19.2)], k.solid, Color32::TRANSPARENT);
        }
        "lineSpacing" => {
            let k = pen.k;
            pen.rows(9.5, 17.0, &[5.0, 10.0, 15.0], k.ink);
            pen.heavy(&[(4.5, 5.0), (4.5, 15.0)], k.solid, 1.1);
            pen.poly(&[(2.0, 5.5), (7.0, 5.5), (4.5, 2.0)], k.solid, Color32::TRANSPARENT);
            pen.poly(&[(2.0, 14.5), (7.0, 14.5), (4.5, 18.0)], k.solid, Color32::TRANSPARENT);
        }
        "shading" => {
            let (k, n) = (pen.k, pen.n);
            pen.poly(&[(4.0, 9.0), (10.0, 3.0), (16.0, 9.0), (10.0, 15.0)], n.tint, n.ink);
            pen.poly(&[(4.0, 9.0), (16.0, 9.0), (10.0, 15.0)], k.tint, n.ink);
            pen.dot(16.8, 13.2, 1.7, k.solid);
            pen.rr(2.0, 16.5, 18.0, 19.2, 0.8, k.solid, Color32::TRANSPARENT);
        }
        "borders" => {
            let k = pen.k;
            for y in [3.0, 10.0, 17.0] {
                for x in [3.0, 6.5, 10.0, 13.5, 17.0] {
                    pen.dot(x, y, 0.7, k.ink);
                }
            }
            for x in [3.0, 10.0, 17.0] {
                for y in [6.5, 13.5] {
                    pen.dot(x, y, 0.7, k.ink);
                }
            }
            pen.heavy(&[(3.0, 17.0), (17.0, 17.0)], k.solid, 1.2);
        }
        // Editing
        "find" => {
            let k = pen.k;
            pen.heavy(&[(12.0, 12.0), (16.8, 16.8)], k.solid, 2.0);
            pen.disc(8.3, 8.3, 5.2, k.tint, k.ink);
        }
        "replace" => {
            let k = pen.k;
            pen.text(6.0, 6.0, 8.5, "ab", k.ink, true);
            pen.text(13.0, 15.0, 8.5, "ac", k.solid, true);
            pen.seg(&[(12.8, 3.5), (16.0, 3.5), (16.0, 8.0)], k.ink);
            pen.poly(&[(14.0, 7.5), (18.0, 7.5), (16.0, 10.0)], k.ink, Color32::TRANSPARENT);
        }
        "select" => {
            let k = pen.k;
            pen.poly(&[(5.0, 3.0), (5.0, 16.0), (8.0, 13.0), (14.0, 12.0)], k.solid, Color32::TRANSPARENT);
            pen.poly(&[(8.0, 13.0), (10.0, 12.0), (12.5, 17.0), (10.5, 18.0)], k.solid, Color32::TRANSPARENT);
            pen.closed(&[(5.0, 3.0), (5.0, 16.0), (8.0, 13.0), (10.5, 18.0), (12.5, 17.0), (10.0, 12.0), (14.0, 12.0)], k.ink);
        }
        "dictate" => {
            let k = pen.k;
            pen.rr(7.3, 2.5, 12.7, 12.0, 2.7, k.solid, k.ink);
            pen.seg(&[(5.0, 9.5), (5.0, 10.5), (10.0, 15.0), (15.0, 10.5), (15.0, 9.5)], k.ink);
            pen.seg(&[(10.0, 15.0), (10.0, 18.0)], k.ink);
            pen.seg(&[(7.5, 18.0), (12.5, 18.0)], k.ink);
        }
        "editor" | "spelling" => {
            let tick = if pen.mono { c } else { tones(Family::Green, false).solid };
            pen.text(8.0, 6.8, 9.0, "abc", pen.k.ink, true);
            pen.heavy(&[(5.0, 13.0), (8.5, 16.5), (16.0, 9.0)], tick, 1.3);
        }
        "styles" => {
            let k = pen.k;
            pen.text(9.0, 9.5, 14.0, "A", k.solid, false);
            pen.seg(&[(13.0, 16.0), (17.5, 11.5)], k.ink);
            pen.dot(13.0, 16.0, 1.2, k.ink);
        }
        "stylesPane" => {
            let (k, n) = (pen.k, pen.n);
            pen.rr(3.0, 3.0, 17.0, 17.0, 1.8, n.tint, n.ink);
            pen.rr(11.0, 3.0, 17.0, 17.0, 1.8, k.tint, n.ink);
            pen.text(7.0, 10.0, 8.5, "A", k.solid, true);
            pen.rows(12.8, 15.2, &[6.5, 9.5, 12.5], k.solid);
        }
        // Insert
        "coverPage" => {
            pen.page(4.0, 2.0, 16.0, 18.0);
            pen.frect(5.5, 7.0, 14.5, 10.0, a);
        }
        "blankPage" => {
            pen.page(4.0, 2.0, 16.0, 18.0);
            pen.line_c(&[(10.0, 7.0), (10.0, 13.0)], a);
            pen.line_c(&[(7.0, 10.0), (13.0, 10.0)], a);
        }
        "pageBreak" => {
            pen.line(&[(4.0, 2.0), (4.0, 7.0), (16.0, 7.0), (16.0, 2.0)]);
            pen.line(&[(4.0, 18.0), (4.0, 13.0), (16.0, 13.0), (16.0, 18.0)]);
            pen.line_c(&[(2.0, 10.0), (18.0, 10.0)], a);
        }
        "drawTable" => {
            pen.rect(2.5, 3.0, 14.0, 14.5, c);
            pen.line(&[(2.5, 8.5), (14.0, 8.5)]);
            pen.line(&[(8.0, 3.0), (8.0, 14.5)]);
            pen.line(&[(9.5, 18.0), (17.0, 10.5), (18.5, 12.0), (11.0, 19.5), (9.0, 19.8), (9.5, 18.0)]);
            pen.fill(&[(17.0, 10.5), (18.0, 9.5), (19.5, 11.0), (18.5, 12.0)], a);
        }
        "table" => {
            pen.frect(3.0, 3.0, 17.0, 7.0, a);
            pen.rect(3.0, 3.0, 17.0, 17.0, c);
            pen.line(&[(3.0, 7.0), (17.0, 7.0)]);
            pen.line(&[(3.0, 12.0), (17.0, 12.0)]);
            pen.line(&[(8.0, 3.0), (8.0, 17.0)]);
            pen.line(&[(12.5, 3.0), (12.5, 17.0)]);
        }
        "picture" => {
            pen.rect(2.5, 4.0, 17.5, 16.0, c);
            pen.fill(&[(4.0, 14.5), (8.5, 8.0), (12.0, 14.5)], a);
            pen.fill(&[(9.5, 14.5), (13.0, 10.0), (16.0, 14.5)], green);
            pen.fcircle(13.5, 7.0, 1.5, yellow);
        }
        "onlinePicture" => {
            art(&pen.at(Rect::from_min_size(r.min, r.size() * 0.8)), "picture");
            pen.circle(15.0, 15.0, 3.5, a);
        }
        "shapes" => {
            pen.rect(3.0, 3.0, 11.0, 11.0, c);
            pen.fcircle(13.0, 13.0, 4.5, a);
        }
        // Shape Effects: a square casting a soft shadow down and to the right.
        "shapeEffects" => {
            for (k, alpha) in [(2.0, 30), (1.0, 50), (0.0, 80)] {
                pen.frect(7.0 - k, 7.0 - k, 18.0 + k, 18.0 + k, Color32::from_rgba_unmultiplied(0, 0, 0, alpha));
            }
            pen.frect(3.0, 3.0, 14.0, 14.0, a);
            pen.rect(3.0, 3.0, 14.0, 14.0, c);
        }
        "icons" => {
            pen.circle(10.0, 10.0, 7.5, c);
            pen.fcircle(7.5, 8.0, 1.0, c);
            pen.fcircle(12.5, 8.0, 1.0, c);
            pen.line_c(&[(6.5, 12.0), (10.0, 14.5), (13.5, 12.0)], a);
        }
        "models3d" => {
            pen.closed(&[(10.0, 2.5), (17.0, 6.0), (17.0, 14.0), (10.0, 17.5), (3.0, 14.0), (3.0, 6.0)], c);
            pen.line_c(&[(3.0, 6.0), (10.0, 9.5), (17.0, 6.0)], a);
            pen.line_c(&[(10.0, 9.5), (10.0, 17.5)], a);
        }
        "smartArt" => {
            pen.frect(7.0, 2.5, 13.0, 6.5, a);
            pen.rect(2.5, 13.0, 8.0, 17.0, c);
            pen.rect(12.0, 13.0, 17.5, 17.0, c);
            pen.line(&[(10.0, 6.5), (10.0, 10.0), (5.0, 10.0), (5.0, 13.0)]);
            pen.line(&[(10.0, 10.0), (15.0, 10.0), (15.0, 13.0)]);
        }
        "addShape" => {
            pen.frect(2.5, 3.0, 9.5, 8.0, a);
            pen.rect(2.5, 12.0, 9.5, 17.0, c);
            pen.line(&[(6.0, 8.0), (6.0, 12.0)]);
            pen.line_c(&[(14.5, 10.5), (14.5, 17.5)], green);
            pen.line_c(&[(11.0, 14.0), (18.0, 14.0)], green);
        }
        "promote" => {
            pen.lines(9.0, 17.5, &[5.0, 10.0, 15.0]);
            pen.line_c(&[(6.5, 7.5), (3.0, 10.0), (6.5, 12.5)], a);
        }
        "demote" => {
            pen.lines(9.0, 17.5, &[5.0, 10.0, 15.0]);
            pen.line_c(&[(3.0, 7.5), (6.5, 10.0), (3.0, 12.5)], a);
        }
        "textPane" => {
            pen.rect(2.5, 3.0, 17.5, 17.0, c);
            pen.line(&[(9.0, 3.0), (9.0, 17.0)]);
            pen.lines(4.0, 7.5, &[6.5, 10.0, 13.5]);
            pen.frect(11.0, 6.0, 16.0, 9.0, a);
            pen.frect(11.0, 11.5, 16.0, 14.5, a);
        }
        "smartArtColors" => {
            pen.fcircle(7.0, 7.5, 4.0, a);
            pen.fcircle(13.0, 7.5, 4.0, orange);
            pen.fcircle(10.0, 13.0, 4.0, green);
            pen.circle(10.0, 10.0, 8.0, c);
        }
        "resetGraphic" => {
            pen.rect(7.5, 6.5, 16.5, 15.5, c);
            pen.frect(9.5, 8.5, 14.5, 13.5, a);
            pen.line(&[(3.5, 13.0), (3.5, 4.0), (11.0, 4.0)]);
            pen.line(&[(1.5, 6.0), (3.5, 4.0), (5.5, 6.0)]);
        }
        "chart" => {
            pen.frect(3.5, 11.0, 6.5, 17.0, a);
            pen.frect(8.5, 6.0, 11.5, 17.0, orange);
            pen.frect(13.5, 9.0, 16.5, 17.0, green);
            pen.line(&[(2.0, 17.5), (18.0, 17.5)]);
        }
        "changeChartType" => {
            pen.frect(3.0, 11.0, 5.5, 17.0, a);
            pen.frect(7.0, 7.0, 9.5, 17.0, orange);
            pen.line(&[(2.0, 17.5), (11.0, 17.5)]);
            pen.fill(&[(15.0, 10.0), (15.0, 5.0), (19.0, 8.5)], green);
            pen.circle(15.0, 10.0, 4.0, c);
            pen.line_c(&[(4.0, 5.0), (9.0, 2.5), (12.0, 4.5)], a);
        }
        "chartData" => {
            pen.rect(2.5, 3.0, 17.5, 17.0, c);
            pen.frect(2.5, 3.0, 17.5, 6.5, a);
            pen.line(&[(2.5, 10.0), (17.5, 10.0)]);
            pen.line(&[(2.5, 13.5), (17.5, 13.5)]);
            pen.line(&[(7.5, 3.0), (7.5, 17.0)]);
            pen.line(&[(12.5, 3.0), (12.5, 17.0)]);
            pen.frect(9.0, 14.5, 11.0, 16.0, orange);
        }
        "chartTitle" => {
            pen.frect(5.0, 2.5, 15.0, 4.5, a);
            pen.frect(4.0, 11.0, 7.0, 17.0, c);
            pen.frect(8.5, 8.0, 11.5, 17.0, c);
            pen.frect(13.0, 12.5, 16.0, 17.0, c);
            pen.line(&[(2.5, 17.5), (17.5, 17.5)]);
        }
        "legend" => {
            for (k, col) in [a, orange, green].into_iter().enumerate() {
                let y = 5.0 + 5.0 * k as f32;
                pen.frect(3.0, y - 1.5, 6.0, y + 1.5, col);
                pen.line(&[(8.0, y), (17.0, y)]);
            }
        }
        "dataLabels" => {
            pen.frect(3.5, 11.0, 7.0, 17.5, a);
            pen.frect(12.0, 8.0, 15.5, 17.5, orange);
            pen.rect(2.5, 5.5, 8.0, 9.0, c);
            pen.rect(11.0, 2.5, 16.5, 6.0, c);
        }
        "screenshot" => {
            pen.line(&[(3.0, 7.0), (3.0, 3.0), (7.0, 3.0)]);
            pen.line(&[(13.0, 3.0), (17.0, 3.0), (17.0, 7.0)]);
            pen.line(&[(17.0, 13.0), (17.0, 17.0), (13.0, 17.0)]);
            pen.line(&[(7.0, 17.0), (3.0, 17.0), (3.0, 13.0)]);
            pen.fcircle(10.0, 10.0, 3.0, a);
        }
        "video" => {
            pen.rect(2.5, 5.0, 14.0, 15.0, c);
            pen.fill(&[(14.0, 10.0), (18.0, 6.5), (18.0, 13.5)], a);
        }
        "link" => {
            pen.closed(
                &[(3.0, 10.0), (7.0, 6.0), (9.5, 6.0), (9.5, 8.0), (7.5, 8.0), (5.0, 10.5), (7.5, 13.0), (9.5, 13.0), (9.5, 15.0), (7.0, 15.0)],
                c,
            );
            pen.line_c(&[(7.5, 10.5), (12.5, 10.5)], a);
            pen.closed(
                &[
                    (17.0, 10.5),
                    (13.0, 14.5),
                    (10.5, 14.5),
                    (10.5, 12.5),
                    (12.5, 12.5),
                    (15.0, 10.0),
                    (12.5, 7.5),
                    (10.5, 7.5),
                    (10.5, 5.5),
                    (13.0, 5.5),
                ],
                c,
            );
        }
        "bookmark" => {
            pen.closed(&[(5.5, 2.5), (14.5, 2.5), (14.5, 17.5), (10.0, 13.5), (5.5, 17.5)], c);
            pen.fill(&[(7.0, 4.0), (13.0, 4.0), (13.0, 8.0), (7.0, 8.0)], a);
        }
        "crossRef" => {
            pen.page(3.0, 2.0, 12.0, 14.0);
            pen.line_c(&[(9.0, 12.0), (17.0, 17.0)], a);
            pen.line_c(&[(14.0, 17.5), (17.0, 17.0), (16.5, 14.0)], a);
        }
        "comment" | "newComment" => {
            pen.closed(&[(2.5, 3.5), (17.5, 3.5), (17.5, 13.5), (9.0, 13.5), (5.0, 17.0), (5.0, 13.5), (2.5, 13.5)], c);
            if name == "newComment" {
                pen.line_c(&[(10.0, 6.0), (10.0, 11.0)], a);
                pen.line_c(&[(7.5, 8.5), (12.5, 8.5)], a);
            } else {
                pen.lines(5.5, 14.5, &[7.0, 10.0]);
            }
        }
        "header" => {
            pen.page(4.0, 2.0, 16.0, 18.0);
            pen.frect(5.5, 5.0, 14.5, 7.5, a);
            pen.lines(6.0, 14.0, &[11.0, 13.5]);
        }
        "footer" => {
            pen.page(4.0, 2.0, 16.0, 18.0);
            pen.lines(6.0, 14.0, &[6.5, 9.0]);
            pen.frect(5.5, 13.0, 14.5, 15.5, a);
        }
        "pageNumber" => {
            pen.page(4.0, 2.0, 16.0, 18.0);
            pen.text(10.0, 13.0, 7.0, "#", a, true);
        }
        "textBox" => {
            pen.rect(3.0, 3.0, 17.0, 17.0, c);
            pen.text(10.0, 10.0, 10.0, "A", a, false);
        }
        "quickParts" => {
            pen.rect(3.0, 3.0, 9.0, 9.0, c);
            pen.frect(11.0, 3.0, 17.0, 9.0, a);
            pen.rect(3.0, 11.0, 9.0, 17.0, c);
            pen.rect(11.0, 11.0, 17.0, 17.0, c);
        }
        "wordArt" => {
            pen.text(10.0, 10.5, 15.0, "A", a, true);
            pen.line_c(&[(3.0, 17.5), (17.0, 17.5)], orange);
        }
        "textFill" => {
            pen.text(10.0, 8.5, 13.0, "A", c, true);
            pen.frect(3.0, 15.0, 17.0, 18.5, a);
        }
        "textOutline" => {
            pen.text(10.0, 8.5, 13.0, "A", a, false);
            pen.rect(3.0, 15.0, 17.0, 18.5, c);
        }
        "transform" => {
            // Letters along an arch.
            let arc: Vec<(f32, f32)> = (0..=12)
                .map(|i| {
                    let t = std::f32::consts::PI * (1.0 - i as f32 / 12.0);
                    (10.0 + 7.5 * t.cos(), 15.0 - 9.0 * t.sin())
                })
                .collect();
            pen.line_c(&arc, a);
            pen.text(10.0, 13.0, 8.0, "A", c, true);
            pen.line(&[(2.5, 17.5), (17.5, 17.5)]);
        }
        "dropCap" => {
            pen.text(6.0, 8.0, 12.0, "A", a, true);
            pen.lines(11.0, 17.0, &[4.0, 8.0, 12.0]);
            pen.lines(3.0, 17.0, &[16.0]);
        }
        "signature" => {
            pen.line_c(&[(3.0, 13.0), (6.0, 7.0), (8.0, 12.0), (11.0, 6.0), (13.0, 12.0)], a);
            pen.line(&[(3.0, 16.5), (17.0, 16.5)]);
        }
        "dateTime" => {
            pen.rect(3.0, 4.0, 17.0, 17.0, c);
            pen.frect(3.0, 4.0, 17.0, 7.5, a);
            pen.lines(5.0, 15.0, &[11.0, 14.0]);
        }
        "object" => {
            pen.rect(3.0, 3.0, 17.0, 17.0, c);
            pen.fcircle(10.0, 10.0, 3.5, a);
        }
        "equation" => pen.text(10.0, 10.0, 17.0, "π", a, false),
        "symbol" => pen.text(10.0, 10.0, 17.0, "Ω", c, false),
        // Draw
        "pen" => {
            pen.line(&[(4.0, 16.0), (14.0, 6.0), (16.0, 8.0), (6.0, 18.0), (3.5, 18.5), (4.0, 16.0)]);
            pen.fill(&[(14.0, 6.0), (15.5, 4.5), (17.5, 6.5), (16.0, 8.0)], a);
        }
        "pencil" => {
            let k = pen.k;
            pen.poly(&[(4.0, 16.0), (14.0, 6.0), (16.0, 8.0), (6.0, 18.0)], k.tint, k.ink);
            pen.poly(&[(4.0, 16.0), (6.0, 18.0), (3.0, 19.0)], k.ink, Color32::TRANSPARENT);
            pen.poly(&[(14.0, 6.0), (15.5, 4.5), (17.5, 6.5), (16.0, 8.0)], a, k.ink);
        }
        "eraser" => {
            pen.closed(&[(3.0, 13.0), (11.0, 5.0), (17.0, 11.0), (11.0, 17.0), (7.0, 17.0)], c);
            pen.fill(&[(3.0, 13.0), (7.0, 9.0), (13.0, 15.0), (11.0, 17.0), (7.0, 17.0)], Color32::from_rgb(0xE8, 0x6F, 0xA0));
        }
        // Ink thickness: three strokes, thin to thick.
        "thickness" => {
            pen.frect(3.0, 4.5, 17.0, 5.5, c);
            pen.frect(3.0, 9.0, 17.0, 11.0, c);
            pen.frect(3.0, 14.0, 17.0, 17.0, a);
        }
        "lasso" => {
            pen.closed(&[(4.0, 7.0), (9.0, 3.0), (16.0, 5.0), (16.0, 11.0), (10.0, 13.0), (4.0, 11.0)], c);
            pen.line_c(&[(6.0, 12.5), (5.0, 17.5)], a);
        }
        "inkToShape" => {
            pen.line_c(&[(3.0, 10.0), (5.0, 6.0), (8.0, 12.0), (10.0, 7.0)], a);
            pen.rect(11.0, 9.0, 17.0, 15.0, c);
        }
        "inkToMath" => {
            pen.line_c(&[(3.0, 12.0), (5.0, 7.0), (7.0, 12.0)], a);
            pen.text(13.5, 10.0, 11.0, "√", c, false);
        }
        "canvas" => {
            pen.rect(2.5, 4.0, 17.5, 16.0, c);
            pen.line_c(&[(5.0, 13.0), (9.0, 8.0), (12.0, 11.0), (15.0, 7.0)], a);
        }
        "replay" => {
            pen.circle(10.0, 10.0, 6.5, c);
            pen.fill(&[(8.5, 7.0), (8.5, 13.0), (13.0, 10.0)], a);
        }
        // Add Pen: a pen with a plus.
        "addPen" => {
            pen.line(&[(3.0, 17.0), (11.0, 9.0), (13.0, 11.0), (5.0, 19.0), (2.5, 19.5), (3.0, 17.0)]);
            pen.fill(&[(11.0, 9.0), (12.5, 7.5), (14.5, 9.5), (13.0, 11.0)], a);
            pen.line_c(&[(15.0, 2.0), (15.0, 8.0)], a);
            pen.line_c(&[(12.0, 5.0), (18.0, 5.0)], a);
        }
        // Ink Replay's controls.
        "replayRewind" => {
            pen.frect(4.0, 5.0, 5.5, 15.0, c);
            pen.fill(&[(16.0, 5.0), (16.0, 15.0), (6.5, 10.0)], c);
        }
        "replayPlay" => pen.fill(&[(6.0, 4.0), (6.0, 16.0), (16.0, 10.0)], a),
        "replayPause" => {
            pen.frect(5.5, 4.5, 8.5, 15.5, a);
            pen.frect(11.5, 4.5, 14.5, 15.5, a);
        }
        "replayForward" => {
            pen.fill(&[(4.0, 5.0), (4.0, 15.0), (13.5, 10.0)], c);
            pen.frect(14.5, 5.0, 16.0, 15.0, c);
        }
        // Design
        "themes" => {
            pen.frect(3.0, 3.0, 10.0, 10.0, a);
            pen.frect(10.0, 3.0, 17.0, 10.0, orange);
            pen.frect(3.0, 10.0, 10.0, 17.0, green);
            pen.frect(10.0, 10.0, 17.0, 17.0, yellow);
        }
        "colors" => {
            pen.fcircle(7.0, 7.5, 3.5, red);
            pen.fcircle(13.0, 7.5, 3.5, a);
            pen.fcircle(10.0, 13.0, 3.5, green);
        }
        "fonts" => pen.text(10.0, 10.0, 13.0, "Aa", a, false),
        "paraSpacing" => {
            pen.lines(8.0, 17.0, &[4.0, 7.0, 13.0, 16.0]);
            pen.line_c(&[(4.0, 8.0), (4.0, 12.0)], a);
            pen.line_c(&[(2.5, 9.5), (4.0, 8.0), (5.5, 9.5)], a);
            pen.line_c(&[(2.5, 10.5), (4.0, 12.0), (5.5, 10.5)], a);
        }
        "effectsDesign" => {
            pen.fcircle(10.0, 10.0, 6.0, a.linear_multiply(0.5));
            pen.circle(10.0, 10.0, 6.0, c);
        }
        "setDefault" => {
            pen.line_c(&[(4.0, 10.5), (8.5, 15.0), (16.5, 5.0)], green);
        }
        "watermark" => {
            pen.page(4.0, 2.0, 16.0, 18.0);
            pen.text(10.0, 11.0, 7.0, "W", a.linear_multiply(0.6), true);
        }
        "pageColor" => {
            pen.page(4.0, 2.0, 16.0, 18.0);
            pen.frect(5.0, 9.0, 15.0, 17.0, a.linear_multiply(0.5));
        }
        "pageBorders" => {
            pen.page(4.0, 2.0, 16.0, 18.0);
            pen.rect(6.0, 6.5, 14.0, 15.5, a);
        }
        // Layout
        "margins" => {
            pen.page(4.0, 2.0, 16.0, 18.0);
            pen.frect(6.0, 4.5, 14.0, 15.5, a.linear_multiply(0.3));
        }
        "orientation" => {
            pen.page(3.0, 3.0, 11.0, 15.0);
            pen.rect(9.0, 10.0, 18.0, 17.0, a);
        }
        "size" => {
            pen.page(5.0, 2.0, 15.0, 18.0);
            pen.line_c(&[(2.5, 2.5), (2.5, 17.5)], a);
            pen.line_c(&[(5.0, 19.0), (15.0, 19.0)], a);
        }
        "columns" => {
            pen.lines(3.0, 9.0, &[4.0, 7.0, 10.0, 13.0, 16.0]);
            pen.lines(11.0, 17.0, &[4.0, 7.0, 10.0, 13.0]);
        }
        "breaks" => {
            pen.page(4.0, 1.5, 16.0, 8.5);
            pen.page(4.0, 11.5, 16.0, 18.5);
            pen.line_c(&[(2.0, 10.0), (18.0, 10.0)], a);
        }
        "lineNumbers" => {
            for (i, y) in [4.0, 8.0, 12.0, 16.0].iter().enumerate() {
                pen.text(4.0, *y, 5.0, &(i + 1).to_string(), a, true);
                pen.line(&[(7.0, *y), (17.0, *y)]);
            }
        }
        "hyphenation" => {
            pen.text(7.0, 7.0, 8.0, "ab-", c, false);
            pen.text(9.0, 14.5, 8.0, "cd", a, false);
        }
        "position" | "wrapText" => {
            pen.lines(3.0, 17.0, &[3.0, 17.0]);
            pen.lines(3.0, 6.0, &[7.0, 10.0, 13.0]);
            pen.frect(7.5, 6.0, 12.5, 14.0, a);
            pen.lines(14.0, 17.0, &[7.0, 10.0, 13.0]);
        }
        "bringForward" => {
            pen.rect(3.0, 3.0, 12.0, 12.0, c);
            pen.frect(8.0, 8.0, 17.0, 17.0, a);
        }
        "sendBackward" => {
            pen.frect(3.0, 3.0, 12.0, 12.0, a);
            pen.rect(8.0, 8.0, 17.0, 17.0, c);
        }
        "selectionPane" => {
            pen.rect(3.0, 3.0, 17.0, 17.0, c);
            pen.lines(6.0, 14.0, &[7.0, 10.0, 13.0]);
        }
        "align" => {
            pen.line_c(&[(3.0, 2.0), (3.0, 18.0)], a);
            pen.rect(4.5, 4.0, 15.0, 8.0, c);
            pen.rect(4.5, 11.0, 11.0, 15.0, c);
        }
        "group" => {
            pen.rect(2.5, 2.5, 17.5, 17.5, a);
            pen.rect(5.0, 5.0, 11.0, 11.0, c);
            pen.fcircle(13.0, 13.0, 3.0, c);
        }
        "rotate" => {
            pen.circle(10.0, 10.0, 6.0, c);
            pen.fill(&[(14.5, 3.0), (17.5, 6.0), (13.5, 7.0)], a);
        }
        "indentLeft" | "indentRight" | "spaceBefore" | "spaceAfter" => {
            pen.lines(3.0, 17.0, &[6.0, 10.0, 14.0]);
            pen.fcircle(if name.contains("Right") { 15.0 } else { 5.0 }, 10.0, 1.6, a);
        }
        // References
        "toc" => {
            pen.page(4.0, 2.0, 16.0, 18.0);
            pen.lines(6.0, 11.0, &[7.0, 10.0, 13.0]);
            for y in [7.0, 10.0, 13.0] {
                pen.fcircle(13.5, y, 0.8, a);
            }
        }
        "addText" => {
            pen.lines(3.0, 13.0, &[5.0, 9.0, 13.0]);
            pen.line_c(&[(15.0, 11.0), (15.0, 17.0)], a);
            pen.line_c(&[(12.0, 14.0), (18.0, 14.0)], a);
        }
        "updateTable" | "update" => {
            pen.circle(10.0, 10.0, 6.0, c);
            pen.fill(&[(14.5, 3.0), (17.5, 6.0), (13.5, 7.0)], green);
        }
        "footnote" => {
            pen.text(6.5, 7.0, 10.0, "ab", c, false);
            pen.text(13.5, 4.5, 6.0, "1", a, true);
            pen.line(&[(3.0, 13.0), (9.0, 13.0)]);
            pen.lines(3.0, 17.0, &[16.0]);
        }
        "endnote" => {
            pen.text(6.5, 7.0, 10.0, "ab", c, false);
            pen.text(13.5, 4.5, 6.0, "i", a, true);
            pen.lines(3.0, 17.0, &[13.0, 16.0]);
        }
        "nextFootnote" => {
            pen.text(7.0, 7.0, 8.0, "ab", c, false);
            pen.text(13.0, 5.0, 6.0, "1", a, true);
            pen.line_c(&[(6.0, 15.0), (14.0, 15.0)], a);
            pen.line_c(&[(11.5, 12.5), (14.0, 15.0), (11.5, 17.5)], a);
        }
        "showNotes" => {
            pen.page(4.0, 2.0, 16.0, 18.0);
            pen.frect(5.5, 13.0, 14.5, 16.5, a.linear_multiply(0.5));
        }
        "researcher" => {
            pen.circle(9.0, 9.0, 5.0, c);
            pen.line_c(&[(12.5, 12.5), (17.0, 17.0)], a);
            pen.lines(6.5, 11.5, &[8.0, 10.0]);
        }
        // Zotero
        "docPrefs" => {
            let (k, n) = (pen.k, pen.n);
            pen.poly(&[(3.5, 2.5), (10.5, 2.5), (14.0, 6.0), (14.0, 17.5), (3.5, 17.5)], n.tint, n.ink);
            for y in [7.0, 9.5] {
                pen.seg(&[(5.5, y), (10.0, y)], n.ink);
            }
            for i in 0..8 {
                let t = std::f32::consts::TAU * i as f32 / 8.0;
                pen.dot(14.0 + 4.0 * t.cos(), 14.0 + 4.0 * t.sin(), 1.3, k.solid);
            }
            pen.disc(14.0, 14.0, 3.2, k.solid, Color32::TRANSPARENT);
            pen.disc(14.0, 14.0, 1.3, n.tint, Color32::TRANSPARENT);
        }
        "unlinkCitations" => {
            // Two chain links pulled apart, with a break between them.
            pen.closed(
                &[(2.5, 12.5), (6.0, 9.0), (8.5, 9.0), (8.5, 10.8), (6.8, 10.8), (4.6, 13.0), (6.8, 15.2), (8.5, 15.2), (8.5, 17.0), (6.0, 17.0)],
                c,
            );
            pen.closed(
                &[(17.5, 7.5), (14.0, 11.0), (11.5, 11.0), (11.5, 9.2), (13.2, 9.2), (15.4, 7.0), (13.2, 4.8), (11.5, 4.8), (11.5, 3.0), (14.0, 3.0)],
                c,
            );
            pen.line_c(&[(9.0, 6.5), (7.5, 5.0)], a);
            pen.line_c(&[(11.0, 13.5), (12.5, 15.0)], a);
            pen.line_c(&[(10.0, 5.5), (10.0, 3.5)], a);
            pen.line_c(&[(10.0, 14.5), (10.0, 16.5)], a);
        }
        "addNote" => {
            pen.closed(&[(3.0, 3.0), (17.0, 3.0), (17.0, 12.5), (12.5, 17.0), (3.0, 17.0)], c);
            pen.line(&[(12.5, 17.0), (12.5, 12.5), (17.0, 12.5)]);
            pen.line_c(&[(6.0, 8.0), (11.0, 8.0)], a);
            pen.line_c(&[(8.5, 5.5), (8.5, 10.5)], a);
            pen.lines(6.0, 11.0, &[13.5]);
        }
        "citation" => {
            pen.text(6.0, 9.0, 14.0, "“", a, true);
            pen.lines(9.0, 17.0, &[6.0, 10.0, 14.0]);
        }
        "sources" | "bibliography" => {
            pen.rect(4.0, 3.0, 9.0, 17.0, c);
            pen.rect(9.0, 3.0, 13.0, 17.0, a);
            pen.closed(&[(13.5, 4.0), (16.5, 3.5), (18.0, 16.5), (15.0, 17.0)], c);
        }
        "caption" => {
            pen.rect(3.0, 2.5, 17.0, 11.5, c);
            pen.fill(&[(4.5, 10.0), (8.0, 5.5), (11.0, 10.0)], a);
            pen.lines(3.0, 13.0, &[15.0, 17.5]);
        }
        "tableOfFigures" => {
            pen.page(4.0, 2.0, 16.0, 18.0);
            pen.frect(6.0, 5.0, 10.0, 8.0, a);
            pen.lines(6.0, 14.0, &[11.0, 14.0]);
        }
        "markEntry" | "index" | "markCitation" | "tableOfAuthorities" => {
            pen.page(4.0, 2.0, 16.0, 18.0);
            pen.text(10.0, 10.0, 8.0, if name == "index" || name == "markEntry" { "A" } else { "§" }, a, true);
        }
        // Mailings
        "envelope" => {
            pen.rect(2.5, 5.0, 17.5, 15.5, c);
            pen.line_c(&[(2.5, 5.0), (10.0, 11.0), (17.5, 5.0)], a);
        }
        "labels" => {
            pen.rect(3.0, 3.0, 17.0, 17.0, c);
            pen.frect(5.0, 5.0, 9.5, 9.0, a);
            pen.frect(10.5, 5.0, 15.0, 9.0, a);
            pen.frect(5.0, 11.0, 9.5, 15.0, a);
            pen.frect(10.5, 11.0, 15.0, 15.0, a);
        }
        "mailMerge" | "recipients" | "editRecipients" => {
            pen.fcircle(7.0, 7.0, 2.5, a);
            pen.line(&[(3.0, 15.0), (3.0, 13.0), (7.0, 10.5), (11.0, 13.0), (11.0, 15.0)]);
            pen.fcircle(13.5, 6.0, 2.0, c);
            pen.line(&[(11.5, 10.0), (13.5, 9.0), (17.0, 11.0), (17.0, 14.0)]);
        }
        "mergeField" | "addressBlock" | "greetingLine" | "rules" | "matchFields" | "highlightFields" => {
            pen.rect(2.5, 6.0, 17.5, 14.0, c);
            pen.text(10.0, 10.0, 7.0, "«»", a, true);
        }
        "preview" => {
            pen.text(10.0, 10.0, 8.0, "ABC", a, true);
            pen.rect(2.5, 5.0, 17.5, 15.0, c);
        }
        "next" | "previous" | "first" | "last" => {
            let pts: &[(f32, f32)] =
                if name == "next" || name == "last" { &[(7.0, 4.0), (13.0, 10.0), (7.0, 16.0)] } else { &[(13.0, 4.0), (7.0, 10.0), (13.0, 16.0)] };
            pen.line_c(pts, a);
        }
        "finish" => {
            pen.page(3.0, 2.0, 13.0, 15.0);
            pen.line_c(&[(10.0, 15.0), (13.0, 18.0), (18.0, 11.0)], green);
        }
        "checkErrors" => {
            pen.page(4.0, 2.0, 16.0, 18.0);
            pen.fcircle(14.0, 14.0, 3.5, red);
        }
        // Review
        "thesaurus" => {
            pen.rect(3.5, 2.5, 16.5, 17.5, c);
            pen.text(10.0, 9.5, 8.0, "Aa", a, true);
        }
        "wordCount" => pen.text(10.0, 10.0, 9.0, "123", a, true),
        "readAloud" => {
            pen.text(6.0, 10.0, 10.0, "A", c, false);
            pen.line_c(&[(12.0, 7.0), (13.5, 10.0), (12.0, 13.0)], a);
            pen.line_c(&[(14.5, 5.0), (17.0, 10.0), (14.5, 15.0)], a);
        }
        "accessibility" => {
            pen.fcircle(10.0, 4.0, 1.6, a);
            pen.line(&[(4.0, 7.0), (16.0, 7.0)]);
            pen.line(&[(10.0, 7.0), (10.0, 12.0), (7.0, 17.5)]);
            pen.line(&[(10.0, 12.0), (13.0, 17.5)]);
        }
        "altText" => {
            // A small picture with lines of text beside and below it.
            pen.rect(2.5, 3.5, 11.5, 11.0, c);
            pen.line_c(&[(3.5, 10.0), (6.0, 6.5), (8.0, 8.5), (9.0, 7.5), (10.5, 10.0)], a);
            for (x0, x1, y) in [(13.5, 17.5, 5.0), (13.5, 17.5, 8.0), (2.5, 17.5, 14.0), (2.5, 13.0, 17.0)] {
                pen.line_c(&[(x0, y), (x1, y)], c);
            }
        }
        "translate" | "language" => {
            pen.text(6.5, 7.0, 9.0, "A", c, true);
            pen.text(13.5, 13.5, 9.0, "あ", a, false);
        }
        "deleteComment" => {
            pen.closed(&[(2.5, 3.5), (17.5, 3.5), (17.5, 13.5), (9.0, 13.5), (5.0, 17.0), (5.0, 13.5), (2.5, 13.5)], c);
            pen.line_c(&[(7.5, 6.0), (12.5, 11.0)], red);
            pen.line_c(&[(12.5, 6.0), (7.5, 11.0)], red);
        }
        "prevComment" | "nextComment" => {
            pen.closed(&[(2.5, 3.5), (17.5, 3.5), (17.5, 13.5), (9.0, 13.5), (5.0, 17.0), (5.0, 13.5), (2.5, 13.5)], c);
            let pts: &[(f32, f32)] =
                if name == "nextComment" { &[(8.0, 6.0), (12.0, 8.5), (8.0, 11.0)] } else { &[(12.0, 6.0), (8.0, 8.5), (12.0, 11.0)] };
            pen.line_c(pts, a);
        }
        "showComments" => {
            pen.rect(3.0, 3.0, 17.0, 17.0, c);
            pen.frect(12.0, 3.0, 17.0, 17.0, a.linear_multiply(0.4));
        }
        "resolve" => {
            pen.closed(&[(2.5, 3.5), (17.5, 3.5), (17.5, 13.5), (9.0, 13.5), (5.0, 17.0), (5.0, 13.5), (2.5, 13.5)], c);
            pen.line_c(&[(6.5, 8.5), (9.0, 11.0), (13.5, 6.0)], green);
        }
        "trackChanges" => {
            pen.page(4.0, 2.0, 16.0, 18.0);
            pen.line_c(&[(6.0, 7.0), (14.0, 7.0)], red);
            pen.line_c(&[(6.0, 11.0), (14.0, 11.0)], a);
            pen.lines(6.0, 11.0, &[15.0]);
        }
        "markup" | "reviewingPane" => {
            pen.page(4.0, 2.0, 16.0, 18.0);
            pen.line_c(&[(6.0, 8.0), (13.0, 8.0)], red);
            pen.lines(6.0, 13.0, &[12.0]);
        }
        "accept" => {
            pen.page(4.0, 2.0, 15.0, 17.0);
            pen.line_c(&[(10.0, 14.5), (12.5, 17.0), (18.0, 11.0)], green);
        }
        "reject" => {
            pen.page(4.0, 2.0, 15.0, 17.0);
            pen.line_c(&[(11.0, 12.0), (17.0, 18.0)], red);
            pen.line_c(&[(17.0, 12.0), (11.0, 18.0)], red);
        }
        "prevChange" | "nextChange" => {
            pen.page(4.0, 2.0, 16.0, 18.0);
            let pts: &[(f32, f32)] =
                if name == "nextChange" { &[(8.5, 7.0), (11.5, 10.0), (8.5, 13.0)] } else { &[(11.5, 7.0), (8.5, 10.0), (11.5, 13.0)] };
            pen.line_c(pts, a);
        }
        "compare" | "combine" => {
            pen.page(2.0, 3.0, 10.0, 15.0);
            pen.page(10.0, 5.0, 18.0, 17.0);
            pen.line_c(&[(6.0, 18.5), (14.0, 18.5)], a);
        }
        "protect" | "restrict" | "blockAuthors" => {
            pen.rect(5.0, 9.0, 15.0, 17.5, c);
            pen.line_c(&[(7.0, 9.0), (7.0, 6.0), (10.0, 3.0), (13.0, 6.0), (13.0, 9.0)], a);
        }
        "hideInk" => {
            pen.line_c(&[(3.0, 13.0), (6.0, 8.0), (9.0, 13.0), (12.0, 8.0)], a);
            pen.line(&[(3.0, 3.0), (17.0, 17.0)]);
        }
        // View
        "readMode" => {
            pen.closed(&[(2.0, 4.0), (9.5, 5.0), (9.5, 17.0), (2.0, 16.0)], c);
            pen.closed(&[(18.0, 4.0), (10.5, 5.0), (10.5, 17.0), (18.0, 16.0)], a);
        }
        "printLayout" => {
            pen.page(4.0, 2.0, 16.0, 18.0);
            pen.lines(6.0, 14.0, &[7.0, 10.0, 13.0]);
        }
        "webLayout" => {
            pen.circle(10.0, 10.0, 7.0, c);
            pen.line_c(&[(3.0, 10.0), (17.0, 10.0)], a);
            pen.line_c(&[(10.0, 3.0), (7.0, 10.0), (10.0, 17.0)], a);
            pen.line_c(&[(10.0, 3.0), (13.0, 10.0), (10.0, 17.0)], a);
        }
        "outline" => {
            pen.fcircle(4.0, 4.5, 1.2, a);
            pen.line(&[(6.5, 4.5), (17.0, 4.5)]);
            pen.fcircle(7.0, 10.0, 1.2, a);
            pen.line(&[(9.5, 10.0), (17.0, 10.0)]);
            pen.fcircle(10.0, 15.5, 1.2, a);
            pen.line(&[(12.5, 15.5), (17.0, 15.5)]);
        }
        "draft" => pen.lines(3.0, 17.0, &[4.0, 7.0, 10.0, 13.0, 16.0]),
        "focus" => {
            pen.line(&[(3.0, 7.0), (3.0, 3.0), (7.0, 3.0)]);
            pen.line(&[(13.0, 3.0), (17.0, 3.0), (17.0, 7.0)]);
            pen.line(&[(17.0, 13.0), (17.0, 17.0), (13.0, 17.0)]);
            pen.line(&[(7.0, 17.0), (3.0, 17.0), (3.0, 13.0)]);
            pen.lines(7.0, 13.0, &[8.5, 11.5]);
        }
        "immersive" => {
            pen.closed(&[(2.0, 4.0), (9.5, 5.0), (9.5, 17.0), (2.0, 16.0)], c);
            pen.closed(&[(18.0, 4.0), (10.5, 5.0), (10.5, 17.0), (18.0, 16.0)], c);
            pen.fcircle(14.0, 10.0, 2.0, a);
        }
        "vertical" => {
            pen.page(5.0, 1.0, 15.0, 9.0);
            pen.page(5.0, 11.0, 15.0, 19.0);
        }
        "sideToSide" => {
            pen.page(1.5, 4.0, 9.5, 16.0);
            pen.page(10.5, 4.0, 18.5, 16.0);
        }
        "ruler" => {
            pen.rect(2.0, 7.0, 18.0, 13.0, c);
            for x in [4.5, 7.0, 9.5, 12.0, 14.5] {
                pen.line_c(&[(x, 7.0), (x, 10.0)], a);
            }
        }
        "gridlines" => {
            pen.rect(3.0, 3.0, 17.0, 17.0, c);
            for k in [6.5, 10.0, 13.5] {
                pen.line_c(&[(k, 3.0), (k, 17.0)], a.linear_multiply(0.6));
                pen.line_c(&[(3.0, k), (17.0, k)], a.linear_multiply(0.6));
            }
        }
        "navPane" => {
            pen.rect(3.0, 3.0, 17.0, 17.0, c);
            pen.frect(3.0, 3.0, 8.0, 17.0, a.linear_multiply(0.5));
        }
        "zoom" => {
            pen.circle(8.5, 8.5, 5.5, c);
            pen.line_c(&[(12.5, 12.5), (17.5, 17.5)], a);
            pen.line(&[(6.0, 8.5), (11.0, 8.5)]);
            pen.line(&[(8.5, 6.0), (8.5, 11.0)]);
        }
        "zoomIn" | "zoomOut" => {
            pen.circle(8.5, 8.5, 5.5, c);
            pen.line_c(&[(12.5, 12.5), (17.5, 17.5)], a);
            pen.line(&[(6.0, 8.5), (11.0, 8.5)]);
            if name == "zoomIn" {
                pen.line(&[(8.5, 6.0), (8.5, 11.0)]);
            }
        }
        "zoom100" => pen.text(10.0, 10.0, 7.5, "100", a, true),
        "onePage" => pen.page(5.0, 2.0, 15.0, 18.0),
        "multiplePages" => {
            pen.page(1.5, 4.0, 9.5, 16.0);
            pen.page(10.5, 4.0, 18.5, 16.0);
        }
        "pageWidth" => {
            pen.page(4.0, 2.0, 16.0, 18.0);
            pen.line_c(&[(1.5, 10.0), (18.5, 10.0)], a);
        }
        "darkMode" => {
            pen.fcircle(10.0, 10.0, 7.0, c);
            pen.fcircle(13.0, 8.0, 6.0, Color32::from_rgba_unmultiplied(255, 255, 255, 230));
        }
        "interfaceTheme" => {
            // A window, its right half dark: light, dark or both (follow the system).
            pen.frect(10.0, 3.5, 17.5, 16.5, c);
            pen.rect(2.5, 3.5, 17.5, 16.5, c);
            pen.line_c(&[(2.5, 6.5), (10.0, 6.5)], a);
        }
        "newWindow" => {
            pen.rect(2.5, 5.0, 13.0, 15.0, c);
            pen.rect(7.0, 2.5, 17.5, 12.5, a);
        }
        "arrangeAll" => {
            pen.rect(3.0, 2.5, 17.0, 9.0, c);
            pen.rect(3.0, 11.0, 17.0, 17.5, a);
        }
        "split" => {
            pen.rect(3.0, 3.0, 17.0, 17.0, c);
            pen.line_c(&[(3.0, 10.0), (17.0, 10.0)], a);
        }
        "fullScreen" => {
            // A screen with arrows out to its four corners.
            pen.rect(2.0, 3.5, 18.0, 16.5, c);
            pen.line_c(&[(5.0, 8.5), (5.0, 6.0), (7.5, 6.0)], a);
            pen.line_c(&[(12.5, 6.0), (15.0, 6.0), (15.0, 8.5)], a);
            pen.line_c(&[(15.0, 11.5), (15.0, 14.0), (12.5, 14.0)], a);
            pen.line_c(&[(7.5, 14.0), (5.0, 14.0), (5.0, 11.5)], a);
            pen.line_c(&[(5.0, 6.0), (8.0, 8.5)], a);
            pen.line_c(&[(15.0, 6.0), (12.0, 8.5)], a);
            pen.line_c(&[(15.0, 14.0), (12.0, 11.5)], a);
            pen.line_c(&[(5.0, 14.0), (8.0, 11.5)], a);
        }
        "sideBySide" | "syncScroll" | "switchWindows" => {
            pen.rect(2.0, 4.0, 9.5, 16.0, c);
            pen.rect(10.5, 4.0, 18.0, 16.0, a);
        }
        "macros" => {
            pen.rect(3.0, 3.0, 17.0, 17.0, c);
            pen.fill(&[(8.0, 6.5), (8.0, 13.5), (14.0, 10.0)], a);
        }
        // A page with a filled band at the top and a style mark: the document's template.
        "documentTemplate" => {
            pen.page(4.0, 2.0, 16.0, 18.0);
            pen.frect(6.0, 5.0, 11.5, 7.0, a);
            pen.text(10.0, 13.0, 8.0, "A", a, false);
        }
        "properties" | "info" => {
            let (bg, fg) = pen.badge();
            pen.disc(10.0, 10.0, 7.5, bg, pen.k.ink);
            pen.dot(10.0, 6.4, 1.2, fg);
            pen.heavy(&[(10.0, 9.2), (10.0, 14.2)], fg, 1.3);
        }
        // Table tools
        "insertAbove" | "insertBelow" | "insertLeft" | "insertRight" => {
            pen.rect(3.0, 3.0, 17.0, 17.0, c);
            match name {
                "insertAbove" => pen.frect(3.0, 3.0, 17.0, 8.0, a),
                "insertBelow" => pen.frect(3.0, 12.0, 17.0, 17.0, a),
                "insertLeft" => pen.frect(3.0, 3.0, 8.0, 17.0, a),
                _ => pen.frect(12.0, 3.0, 17.0, 17.0, a),
            }
        }
        "deleteTable" => {
            pen.rect(3.0, 3.0, 15.0, 15.0, c);
            pen.line(&[(3.0, 9.0), (15.0, 9.0)]);
            pen.line(&[(9.0, 3.0), (9.0, 15.0)]);
            pen.line_c(&[(12.0, 12.0), (18.0, 18.0)], red);
            pen.line_c(&[(18.0, 12.0), (12.0, 18.0)], red);
        }
        "merge" => {
            pen.rect(3.0, 5.0, 17.0, 15.0, c);
            pen.line_c(&[(5.0, 10.0), (15.0, 10.0)], a);
            pen.line_c(&[(7.0, 8.0), (5.0, 10.0), (7.0, 12.0)], a);
            pen.line_c(&[(13.0, 8.0), (15.0, 10.0), (13.0, 12.0)], a);
        }
        "splitCells" | "splitTable" => {
            pen.rect(3.0, 5.0, 17.0, 15.0, c);
            pen.line_c(&[(10.0, 5.0), (10.0, 15.0)], a);
        }
        "autofit" | "distributeRows" | "distributeCols" | "rowHeight" | "colWidth" => {
            pen.rect(3.0, 4.0, 17.0, 16.0, c);
            pen.line_c(&[(5.0, 10.0), (15.0, 10.0)], a);
            pen.line_c(&[(7.0, 8.0), (5.0, 10.0), (7.0, 12.0)], a);
            pen.line_c(&[(13.0, 8.0), (15.0, 10.0), (13.0, 12.0)], a);
        }
        "cellAlign" => {
            pen.rect(3.0, 3.0, 17.0, 17.0, c);
            pen.lines(6.0, 14.0, &[8.5, 11.5]);
        }
        "textDirection" => {
            pen.text(9.0, 10.0, 10.0, "A", c, false);
            pen.line_c(&[(15.5, 4.0), (15.5, 16.0)], a);
            pen.line_c(&[(13.5, 14.0), (15.5, 16.0), (17.5, 14.0)], a);
        }
        "cellMargins" => {
            pen.rect(3.0, 3.0, 17.0, 17.0, c);
            pen.rect(6.0, 6.0, 14.0, 14.0, a);
        }
        "repeatHeader" => {
            pen.frect(3.0, 3.0, 17.0, 7.0, a);
            pen.rect(3.0, 3.0, 17.0, 17.0, c);
            pen.line(&[(3.0, 12.0), (17.0, 12.0)]);
        }
        "toText" => {
            pen.rect(3.0, 3.0, 11.0, 11.0, c);
            pen.lines(9.0, 17.0, &[14.0, 17.0]);
        }
        "formula" => pen.text(10.0, 10.0, 12.0, "fx", a, false),
        "selectTable" => {
            pen.rect(3.0, 3.0, 17.0, 17.0, c);
            pen.fill(&[(8.0, 7.0), (8.0, 15.0), (10.0, 13.0), (11.5, 16.0), (12.5, 15.5), (11.0, 12.5), (13.5, 12.5)], a);
        }
        "draw" => {
            pen.rect(3.0, 3.0, 13.0, 13.0, c);
            pen.line_c(&[(10.0, 17.0), (17.0, 10.0)], a);
        }
        "borderPainter" => {
            pen.line(&[(4.0, 16.0), (14.0, 6.0), (16.0, 8.0), (6.0, 18.0), (3.5, 18.5), (4.0, 16.0)]);
            pen.line_c(&[(3.0, 3.0), (11.0, 3.0)], a);
        }
        // Chrome
        "save" => {
            let (k, n) = (pen.k, pen.n);
            pen.poly(&[(3.0, 4.5), (4.5, 3.0), (14.0, 3.0), (17.0, 6.0), (17.0, 15.5), (15.5, 17.0), (4.5, 17.0), (3.0, 15.5)], k.tint, k.ink);
            pen.rr(6.0, 3.0, 13.0, 7.5, 0.8, k.solid, Color32::TRANSPARENT);
            pen.rr(5.8, 10.8, 14.2, 17.0, 1.0, n.tint, k.ink);
            pen.seg(&[(8.0, 13.5), (12.0, 13.5)], k.ink);
        }
        "undo" => {
            let k = pen.k;
            let arc: Vec<(f32, f32)> = (0..=12)
                .map(|i| {
                    let t = -std::f32::consts::FRAC_PI_2 + std::f32::consts::PI * i as f32 / 12.0;
                    (12.5 + 4.5 * t.cos(), 12.5 + 4.5 * t.sin())
                })
                .collect();
            pen.seg(&[(6.5, 8.0), (12.5, 8.0)], k.ink);
            pen.seg(&arc, k.ink);
            pen.seg(&[(12.5, 17.0), (8.0, 17.0)], k.ink);
            pen.poly(&[(2.5, 8.0), (7.0, 4.0), (7.0, 12.0)], k.solid, Color32::TRANSPARENT);
        }
        "redo" => {
            let k = pen.k;
            let arc: Vec<(f32, f32)> = (0..=12)
                .map(|i| {
                    let t = std::f32::consts::FRAC_PI_2 + std::f32::consts::PI * i as f32 / 12.0;
                    (7.5 + 4.5 * t.cos(), 12.5 - 4.5 * t.sin())
                })
                .collect();
            pen.seg(&[(13.5, 8.0), (7.5, 8.0)], k.ink);
            pen.seg(&arc, k.ink);
            pen.seg(&[(7.5, 17.0), (12.0, 17.0)], k.ink);
            pen.poly(&[(17.5, 8.0), (13.0, 4.0), (13.0, 12.0)], k.solid, Color32::TRANSPARENT);
        }
        "search" => {
            let k = pen.k;
            pen.heavy(&[(12.2, 12.2), (16.8, 16.8)], k.solid, 2.0);
            pen.disc(8.5, 8.5, 5.2, k.tint, k.ink);
        }
        "share" => {
            let k = pen.k;
            pen.seg(&[(6.0, 9.0), (4.0, 9.0), (4.0, 16.5), (16.0, 16.5), (16.0, 9.0), (14.0, 9.0)], k.ink);
            pen.heavy(&[(10.0, 13.0), (10.0, 5.0)], k.solid, 1.3);
            pen.poly(&[(6.5, 6.5), (10.0, 2.5), (13.5, 6.5)], k.solid, Color32::TRANSPARENT);
        }
        "user" => {
            let k = pen.k;
            pen.disc(10.0, 6.8, 3.6, k.solid, Color32::TRANSPARENT);
            pen.p.add(Shape::convex_polygon(
                (0..=16)
                    .map(|i| {
                        let t = std::f32::consts::PI * i as f32 / 16.0;
                        pen.pt(10.0 - 6.5 * t.cos(), 18.0 - 6.0 * t.sin())
                    })
                    .collect(),
                k.tint,
                pen.stroke(k.ink),
            ));
        }
        "close" => {
            pen.seg(&[(5.5, 5.5), (14.5, 14.5)], pen.n.ink);
            pen.seg(&[(14.5, 5.5), (5.5, 14.5)], pen.n.ink);
        }
        // A triangle filling most of its box: it is drawn at 9–12 pt beside combos and split
        // buttons, where a smaller one shrank to a dot (#535).
        "dropdown" => pen.fill(&[(3.0, 7.0), (17.0, 7.0), (10.0, 14.0)], c),
        "more" => {
            for x in [5.0, 10.0, 15.0] {
                pen.dot(x, 10.0, 1.5, pen.n.ink);
            }
        }
        "launcher" => {
            let n = pen.n.ink;
            pen.seg(&[(5.0, 5.0), (5.0, 15.0), (15.0, 15.0)], n);
            pen.seg(&[(8.0, 8.0), (14.5, 14.5)], n);
            pen.seg(&[(11.0, 15.0), (15.0, 15.0), (15.0, 11.0)], n);
        }
        "pin" => {
            let k = pen.k;
            pen.seg(&[(10.0, 12.0), (10.0, 18.0)], k.ink);
            pen.rr(7.0, 2.5, 13.0, 9.5, 1.5, k.solid, Color32::TRANSPARENT);
            pen.seg(&[(5.0, 12.0), (15.0, 12.0)], k.ink);
            pen.poly(&[(7.0, 9.0), (13.0, 9.0), (14.5, 12.0), (5.5, 12.0)], k.tint, k.ink);
        }
        "plus" => {
            pen.seg(&[(10.0, 4.5), (10.0, 15.5)], pen.n.ink);
            pen.seg(&[(4.5, 10.0), (15.5, 10.0)], pen.n.ink);
        }
        "minus" => pen.seg(&[(4.5, 10.0), (15.5, 10.0)], pen.n.ink),
        "check" => pen.seg(&[(4.5, 10.5), (8.5, 14.5), (16.0, 5.5)], pen.n.ink),
        "chevronLeft" => pen.seg(&[(12.5, 4.5), (7.0, 10.0), (12.5, 15.5)], pen.n.ink),
        "chevronRight" => pen.seg(&[(7.5, 4.5), (13.0, 10.0), (7.5, 15.5)], pen.n.ink),
        "chevronUp" => pen.seg(&[(4.5, 12.5), (10.0, 7.0), (15.5, 12.5)], pen.n.ink),
        "chevronDown" => pen.seg(&[(4.5, 7.5), (10.0, 13.0), (15.5, 7.5)], pen.n.ink),
        // Read Aloud player
        "mediaPlay" => pen.fill(&[(6.0, 4.0), (16.0, 10.0), (6.0, 16.0)], c),
        "mediaPause" => {
            pen.frect(5.5, 4.0, 8.5, 16.0, c);
            pen.frect(11.5, 4.0, 14.5, 16.0, c);
        }
        "mediaStop" => pen.frect(5.0, 5.0, 15.0, 15.0, c),
        "mediaPrev" => {
            pen.frect(4.0, 5.0, 6.0, 15.0, c);
            pen.fill(&[(16.0, 5.0), (7.0, 10.0), (16.0, 15.0)], c);
        }
        "mediaNext" => {
            pen.frect(14.0, 5.0, 16.0, 15.0, c);
            pen.fill(&[(4.0, 5.0), (13.0, 10.0), (4.0, 15.0)], c);
        }
        // Move past citation: a bracketed citation with an arrow hopping over it to the caret.
        "pastCitation" => {
            pen.line_c(&[(4.5, 9.0), (3.0, 9.0), (3.0, 17.0), (4.5, 17.0)], c);
            pen.line_c(&[(11.5, 9.0), (13.0, 9.0), (13.0, 17.0), (11.5, 17.0)], c);
            pen.lines(5.5, 10.5, &[12.0, 14.5]);
            pen.line_c(&[(4.0, 6.0), (8.0, 2.5), (14.0, 2.5), (16.5, 7.0)], a);
            pen.line_c(&[(14.0, 6.0), (16.5, 7.0), (17.5, 4.5)], a);
            pen.line_c(&[(17.0, 9.0), (17.0, 17.5)], a);
        }
        "chevronDoubleRight" => {
            pen.line(&[(4.0, 4.0), (9.0, 10.0), (4.0, 16.0)]);
            pen.line(&[(11.0, 4.0), (16.0, 10.0), (11.0, 16.0)]);
        }
        "addins" => {
            pen.rect(3.0, 3.0, 9.0, 9.0, c);
            pen.rect(11.0, 3.0, 17.0, 9.0, c);
            pen.rect(3.0, 11.0, 9.0, 17.0, c);
            pen.line_c(&[(14.0, 11.0), (14.0, 17.0)], a);
            pen.line_c(&[(11.0, 14.0), (17.0, 14.0)], a);
        }
        "discord" => {
            pen.fill(&[(3.0, 6.0), (7.0, 3.5), (13.0, 3.5), (17.0, 6.0), (17.5, 14.0), (13.5, 16.5), (6.5, 16.5), (2.5, 14.0)], a);
            pen.fcircle(7.5, 10.5, 1.5, Color32::WHITE);
            pen.fcircle(12.5, 10.5, 1.5, Color32::WHITE);
        }
        // Open: a folder with its flap lifted.
        "open" => {
            let k = pen.k;
            pen.poly(&[(2.5, 5.0), (3.5, 4.0), (7.5, 4.0), (9.0, 5.5), (15.5, 5.5), (16.5, 6.5), (16.5, 15.5), (2.5, 15.5)], k.tint, k.ink);
            pen.poly(&[(5.0, 9.0), (18.5, 9.0), (16.0, 16.0), (2.5, 16.0)], k.solid, k.ink);
        }
        // Print: a printer with a sheet coming out.
        "print" => {
            let (k, n) = (pen.k, pen.n);
            pen.rr(6.0, 2.5, 14.0, 7.5, 0.8, n.tint, n.ink);
            pen.rr(2.5, 7.0, 17.5, 14.5, 2.0, k.tint, k.ink);
            pen.rr(6.0, 11.5, 14.0, 17.5, 0.8, n.tint, n.ink);
            pen.seg(&[(8.0, 14.0), (12.0, 14.0)], k.solid);
            pen.dot(14.8, 9.6, 1.0, k.solid);
        }
        "alignText" => {
            // A box with its text lines in the middle, arrows to the top and bottom edges.
            pen.rect(3.0, 2.5, 17.0, 17.5, c);
            pen.lines(5.5, 11.0, &[9.0, 11.5]);
            pen.line_c(&[(14.0, 5.0), (14.0, 15.0)], a);
            pen.line_c(&[(12.5, 6.5), (14.0, 5.0), (15.5, 6.5)], a);
            pen.line_c(&[(12.5, 13.5), (14.0, 15.0), (15.5, 13.5)], a);
        }
        "textBoxLink" | "textBoxUnlink" => {
            // Two boxes, the text running from the first into the second (broken: cut through).
            pen.rect(2.0, 3.0, 8.5, 11.0, c);
            pen.rect(11.5, 9.0, 18.0, 17.0, c);
            pen.lines(3.5, 7.0, &[5.5, 8.0]);
            pen.line_c(&[(5.0, 11.0), (5.0, 14.0), (11.5, 14.0)], a);
            pen.line_c(&[(9.5, 12.5), (11.5, 14.0), (9.5, 15.5)], a);
            if name == "textBoxUnlink" {
                pen.line_c(&[(6.5, 16.5), (9.0, 11.5)], c);
            }
        }

        // The built-in file browser (#576): a folder, a file and the home folder.
        "folder" => {
            let k = pen.k;
            pen.poly(&[(2.5, 5.0), (3.5, 4.0), (7.5, 4.0), (9.0, 5.5), (16.5, 5.5), (17.5, 6.5), (17.5, 16.0), (2.5, 16.0)], k.tint, k.ink);
            pen.rr(2.5, 8.0, 17.5, 16.0, 1.2, k.solid, k.ink);
        }
        "file" => {
            let (k, n) = (pen.k, pen.n);
            pen.poly(&[(4.0, 2.5), (12.0, 2.5), (16.0, 6.5), (16.0, 17.5), (4.0, 17.5)], n.tint, n.ink);
            pen.poly(&[(12.0, 2.5), (16.0, 6.5), (12.0, 6.5)], k.tint, n.ink);
            for y in [9.5, 12.0, 14.5] {
                pen.seg(&[(6.5, y), (13.5, y)], k.solid);
            }
        }
        "homeFolder" => {
            let k = pen.k;
            pen.rr(4.5, 9.0, 15.5, 17.0, 1.0, k.tint, k.ink);
            pen.poly(&[(10.0, 2.5), (18.0, 9.5), (2.0, 9.5)], k.solid, k.ink);
            pen.rr(8.5, 12.0, 11.5, 17.0, 0.6, k.ink, Color32::TRANSPARENT);
        }

        // Outlining tab: arrows moving a paragraph between outline levels and along the outline.
        "outlinePromote" | "outlineDemote" => {
            pen.lines(9.0, 17.0, &[6.0, 10.0, 14.0]);
            if name == "outlinePromote" {
                pen.line_c(&[(7.0, 10.0), (2.5, 10.0)], a);
                pen.line_c(&[(5.0, 7.5), (2.5, 10.0), (5.0, 12.5)], a);
            } else {
                pen.line_c(&[(2.5, 10.0), (7.0, 10.0)], a);
                pen.line_c(&[(4.5, 7.5), (7.0, 10.0), (4.5, 12.5)], a);
            }
        }
        "outlineTop" | "outlineBody" => {
            pen.lines(11.0, 17.0, &[6.0, 10.0, 14.0]);
            let (x, d) = if name == "outlineTop" { (3.0, 1.0) } else { (9.0, -1.0) };
            for k in [0.0, 3.5] {
                pen.line_c(&[(x + d * k + d * 2.5, 7.0), (x + d * k, 10.0), (x + d * k + d * 2.5, 13.0)], a);
            }
        }
        "outlineUp" | "outlineDown" => {
            pen.lines(3.0, 11.0, &[6.0, 10.0, 14.0]);
            let up = name == "outlineUp";
            let (tip, tail) = if up { (3.0, 17.0) } else { (17.0, 3.0) };
            pen.line_c(&[(15.0, tail), (15.0, tip)], a);
            let wing = if up { tip + 3.0 } else { tip - 3.0 };
            pen.line_c(&[(12.0, wing), (15.0, tip), (18.0, wing)], a);
        }
        "outlineExpand" | "outlineCollapse" => {
            pen.fcircle(10.0, 10.0, 6.5, c);
            pen.line_c(&[(6.5, 10.0), (13.5, 10.0)], Color32::WHITE);
            if name == "outlineExpand" {
                pen.line_c(&[(10.0, 6.5), (10.0, 13.5)], Color32::WHITE);
            }
        }
        "closeOutline" => {
            pen.page(4.0, 2.0, 16.0, 18.0);
            pen.line_c(&[(7.0, 8.0), (13.0, 14.0)], red);
            pen.line_c(&[(13.0, 8.0), (7.0, 14.0)], red);
        }
        "help" => {
            let (bg, fg) = pen.badge();
            pen.disc(10.0, 10.0, 7.5, bg, pen.k.ink);
            pen.text(10.0, 9.6, 11.0, "?", fg, true);
        }
        // Developer › Controls: a field's frame with what it holds.
        "ccRichText" => {
            pen.rect(2.5, 5.0, 17.5, 15.0, a);
            pen.text(7.5, 10.0, 8.0, "A", c, true);
            pen.text(13.0, 10.5, 7.0, "a", c, false);
        }
        "ccPlainText" => {
            pen.rect(2.5, 5.0, 17.5, 15.0, a);
            pen.text(8.0, 10.0, 8.0, "A", c, false);
            pen.line(&[(12.0, 7.5), (12.0, 12.5)]);
        }
        "ccPicture" => {
            pen.rect(2.5, 4.0, 17.5, 16.0, a);
            pen.line(&[(4.5, 14.0), (8.5, 9.5), (11.5, 12.5), (13.5, 10.5), (15.5, 14.0)]);
            pen.fcircle(13.5, 7.0, 1.2, c);
        }
        "ccGallery" => {
            pen.rect(2.5, 3.0, 17.5, 17.0, a);
            pen.frect(5.0, 5.5, 9.0, 9.5, c);
            pen.frect(11.0, 5.5, 15.0, 9.5, c);
            pen.lines(5.0, 15.0, &[12.5, 15.0]);
        }
        "ccCheckBox" => {
            pen.rect(4.0, 4.0, 16.0, 16.0, c);
            pen.line_c(&[(6.5, 10.0), (9.0, 13.0), (14.0, 6.5)], a);
        }
        "ccComboBox" => {
            pen.rect(2.5, 6.0, 17.5, 14.0, c);
            pen.line(&[(13.0, 6.0), (13.0, 14.0)]);
            pen.lines(4.5, 10.5, &[10.0]);
            pen.arrow_down(15.25, 10.0, a);
        }
        "ccDropDown" => {
            pen.rect(2.5, 3.0, 17.5, 9.0, c);
            pen.line(&[(13.0, 3.0), (13.0, 9.0)]);
            pen.arrow_down(15.25, 6.0, a);
            pen.rect(2.5, 9.0, 13.0, 17.0, c);
            pen.lines(4.5, 11.0, &[11.5, 14.5]);
        }
        "ccDate" => {
            pen.rect(3.0, 4.5, 17.0, 17.0, c);
            pen.frect(3.0, 4.5, 17.0, 8.0, a);
            pen.line(&[(6.5, 3.0), (6.5, 6.0)]);
            pen.line(&[(13.5, 3.0), (13.5, 6.0)]);
            for (x, y) in [(6.5, 11.0), (10.0, 11.0), (13.5, 11.0), (6.5, 14.5), (10.0, 14.5)] {
                pen.fcircle(x, y, 0.9, c);
            }
        }
        "ccRepeat" => {
            pen.rect(2.5, 3.0, 17.5, 17.0, a);
            pen.rect(5.0, 5.5, 15.0, 9.5, c);
            pen.rect(5.0, 11.0, 15.0, 15.0, c);
            pen.line_c(&[(10.0, 15.0), (10.0, 18.5)], a);
        }
        "designMode" => {
            pen.line(&[(4.0, 16.0), (16.0, 16.0), (4.0, 4.0), (4.0, 16.0)]);
            pen.line_c(&[(4.0, 12.0), (8.0, 16.0)], a);
            pen.line(&[(7.0, 13.0), (7.0, 12.0)]);
        }
        _ => return false,
    }
    true
}

/// Draw an Insert › Icons library icon into `r` in `c`, from the same path data the inserted
/// picture is rendered from (`wordcraft_render::icon_lib`).
pub fn library(p: &Painter, r: Rect, icon: &wordcraft_render::icon_lib::Icon, c: Color32) {
    use wordcraft_render::icon_lib::{GRID, STROKE};
    let s = r.width().min(r.height()) / GRID;
    let o = r.center() - vec2(GRID / 2.0 * s, GRID / 2.0 * s);
    let stroke = Stroke::new(STROKE * s, c);
    for piece in icon.pieces() {
        let pts: Vec<Pos2> = piece.points.iter().map(|q| pos2(o.x + q[0] * s, o.y + q[1] * s)).collect();
        if piece.fill {
            p.add(Shape::convex_polygon(pts, c, Stroke::NONE));
        } else if piece.closed {
            p.add(Shape::closed_line(pts, stroke));
        } else {
            // Round caps, as on the inserted picture.
            for end in [pts.first(), pts.last()].into_iter().flatten() {
                p.circle_filled(*end, stroke.width / 2.0, c);
            }
            p.add(Shape::line(pts, stroke));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Paint `name` and return the unmultiplied RGB of every visible fill, stroke and text colour.
    fn colours(name: &str, c: Color32, a: Color32) -> Vec<[u8; 3]> {
        let ctx = egui::Context::default();
        ctx.begin_pass(Default::default());
        crate::theme::install_fonts(&ctx);
        ctx.end_pass().textures_delta.clear();
        ctx.begin_pass(Default::default());
        let p = ctx.layer_painter(egui::LayerId::background());
        paint(&p, Rect::from_min_size(pos2(0.0, 0.0), vec2(32.0, 32.0)), name, c, a);
        let mut out = ctx.end_pass();
        out.textures_delta.clear();
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
                Shape::Rect(r) => {
                    push(r.fill);
                    push(r.stroke.color);
                }
                Shape::Circle(c) => {
                    push(c.fill);
                    push(c.stroke.color);
                }
                Shape::Path(pth) => {
                    push(pth.fill);
                    if let egui::epaint::ColorMode::Solid(c) = pth.stroke.color {
                        push(c);
                    }
                }
                Shape::LineSegment { stroke, .. } => push(stroke.color),
                Shape::Text(t) => {
                    push(t.fallback_color);
                    for r in &t.galley.job.sections {
                        push(r.format.color);
                    }
                }
                _ => {}
            }
        }
        for cs in &out.shapes {
            walk(&cs.shape, &mut v);
        }
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
        for name in [
            "paste",
            "save",
            "table",
            "picture",
            "comment",
            "bold",
            "cut",
            "copy",
            "painter",
            "underline",
            "strike",
            "charborder",
            "enclose",
            "case",
            "clear",
            "highlight",
            "fontcolor",
            "effects",
            "bullets",
            "numbering",
            "multilevel",
            "indent",
            "sort",
            "alignLeft",
            "justify",
            "textLtr",
            "lineSpacing",
            "shading",
            "borders",
            "find",
            "replace",
            "select",
            "dictate",
            "spelling",
            "styles",
            "stylesPane",
            "info",
            "help",
            "homeFolder",
            "docPrefs",
            "open",
            "folder",
            "print",
            "pin",
            "user",
            "share",
            "search",
            "undo",
        ] {
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
        for name in ["pen", "pencil"] {
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
}
