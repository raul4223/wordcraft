# Colourful icon set: design

Status: direction approved in chat 2026-10-14 (palette A "colour by function", hand redraw in
batches), awaiting spec review. Sub-project 2 of 3 (after the start page, before AI agents).

## Goal

Replace the thin one-colour line icons (306 names in `crates/ui-egui/src/icons.rs`) with a modern,
colourful, production-looking set: two-tone filled shapes, rounded corners, slightly heavier
strokes, one colour family per function. Every icon keeps its name and so its command; nothing
about behaviour changes. All artwork stays original and drawn in code (no Microsoft assets).

Success: the ribbon, Backstage, panes and dialogs read as a coherent colourful set in light and
dark themes at 16 and 32 pt; disabled and monochrome uses still render correctly; the icon
contact sheet shows no letter-tile fallbacks for names that had real art.

## Palette (colour by function)

Each family has three tones per theme: **ink** (outline/linework), **tint** (light fill) and
**solid** (strong fill). Dark theme uses lighter ink, darker tint and brighter solid.

| Family | Use | Light ink / tint / solid | Dark ink / tint / solid |
|---|---|---|---|
| Neutral | pages, paper, generic linework | `#424242` / `#FFFFFF` / `#6B6B6B` | `#DDDDDD` / `#3A3A3A` / `#B5B5B5` |
| Orange | clipboard, drawing tools, shapes | `#7A4A12` / `#FBE3C2` / `#E8891C` | `#F6C48A` / `#5A3A14` / `#F29A35` |
| Red | font, delete/reject, errors | `#7A1F1F` / `#FBD5D5` / `#D93636` | `#F5A8A8` / `#5C2222` / `#F05A5A` |
| Blue | paragraph, editing, file/chrome, view | `#1F3F73` / `#D6E6FB` / `#2F7BE0` | `#B9D3F7` / `#1D3A5E` / `#5A9BF0` |
| Green | insert, tables, accept/confirm | `#1E6B3A` / `#D7F1DF` / `#2EA35A` | `#9FDDB4` / `#1D4A2C` / `#45C273` |
| Purple | review, comments, styles/design | `#4B2F86` / `#E7DDFB` / `#7C4FD9` | `#CDB8F5` / `#3A2766` / `#9B76EE` |
| Teal | layout, references | `#0F5C55` / `#D2F0EC` / `#1C9C8F` | `#9ADED6` / `#16443F` / `#3CC2B3` |
| Amber | mailings, highlights, folders, sun/star details | `#6E5200` / `#FDEFC3` / `#F2B705` | `#F5DA8A` / `#4D3D0E` / `#F5C842` |

Family per icon is a table in code (`fn family(name) -> Family`), grouped as in the ribbon:
Clipboard → orange; Font → red (except `highlight` amber); Paragraph, Editing, chrome
(save/undo/redo/search/share/user), View → blue; Insert, Table Design/Layout → green; Draw,
shapes, WordArt → orange; Design → purple; Layout → teal; References → teal; Mailings → amber;
Review, comments → purple (accept green, reject/delete red); Read Aloud/media → blue; generic
controls (`dropdown`, `chevron*`, `plus`, `minus`, `check`, `close`, `more`, `launcher`) stay
**neutral**, so small controls don't turn into confetti.

## Rendering rules (unchanged API)

`icons::paint(p, r, name, c, accent)` keeps its signature; all 41 call sites keep working.

- **Theme** is inferred from `c`'s luminance: light ink (luminance > 0.5) means dark theme.
- **Monochrome mode** when `accent == c` or `accent` is fully transparent: every part is drawn in
  `c` (fills at reduced alpha). This is how callers already ask for disabled icons, the white
  sidebar icons, dropdown arrows and tinted pins, so they keep working with no call-site change.
- **Data-colour icons** keep `accent` for the part that shows the user's chosen colour:
  `pen` and `pencil` (the Draw tab passes the pen's colour). Font colour and highlight buttons
  paint the chosen colour over the icon themselves, so those icons use the family palette.
- Otherwise colours come from the family palette; `accent` is ignored.
- Stroke width 1.4 (on the 20-unit grid) with round joins/caps; rectangles get 1.5–2 unit radii.

## Pen additions

`Pen` gains: palette tones (`ink`, `tint`, `solid`, `neutral_*`), `rrect(x0,y0,x1,y1,radius,fill,stroke)`,
`round_line` (round caps), `poly_fill_stroke` (filled polygon with outline), and `mono` handling
so monochrome mode is one place, not 306.

## Rollout batches (one commit each, contact sheet + ribbon screenshot reviewed per batch)

1. Infrastructure + chrome/generic (Pen, palette, families, monochrome, save/undo/redo/search/share/user/controls/backstage icons).
2. Home tab: clipboard, font, paragraph, styles, editing.
3. Insert tab (pages, tables, illustrations, media, links, comments, header/footer, text, symbols).
4. Draw + Design + Layout.
5. References + Mailings.
6. Review + View + table tabs + content controls + Read Aloud/media + remaining.

## Testing

- Unit tests in `icons.rs`: monochrome mode draws only `c`-coloured shapes (inspect `Shape`s on a
  test painter via `egui::Context` + `layer_painter` output); every name in the contact-sheet list
  has a family; data-colour icons use `accent`; theme detection by luminance.
- Existing UI tests stay green (icons don't affect labels).
- `icon_sheet` example (light and dark) and `ui_shot` ribbon screenshots per batch, compared by eye.
- `cargo xtask ci` (except the known upstream Turkish proofing failure) before each commit.

## Out of scope

Insert › Icons library artwork (`icons::library`), app icon, cursors.
