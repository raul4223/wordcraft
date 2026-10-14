# Start page redesign (File › Home): design

Status: approved in chat 2026-10-14, awaiting spec review. Sub-project 1 of 3 (then: colourful
icon set, then AI agents via MCP + in-app AI pane).

## Goal

File › Home today is a greeting (untranslated), four template thumbnails and a plain Recent list.
Make it feel as alive and useful as a modern office start page, without copying any Word artwork:
everything is drawn in code with original shapes, colours come from `theme::Tokens`.

Success: a user can find, open, pin and manage documents and start from a template from Home
without visiting another page, with hover/animation feedback on every interactive element; all
existing Backstage behaviour and tests keep working.

## Scope

In:
1. Backstage sidebar items get an icon left of the label (existing `icons::paint` names) and an
   animated hover/active fill (`ctx.animate_bool`).
2. Greeting translated (`Good morning/afternoon/evening` added to `pt-br.tsv`; other catalogs that
   miss them fall back to English as today).
3. One search box at the top right of Home, filtering templates *and* recent/pinned documents by
   file name or folder (case-insensitive). Empty query shows everything.
4. Template row: all five templates in one row (wraps on narrow windows). Cards lift on hover
   (2 pt upward offset, soft shadow, accent border), animated. The blank card shows a large `+`
   over its thumbnail. "More templates →" stays and goes to New.
5. Document list with two tabs, **Recent** and **Pinned**, and an "Open file…" button on the
   same row (runs `ui.openFileDialog`). Columns: type icon tinted by extension (docx/doc/dotx,
   md/txt, rtf/odt, html, pdf, other), name, folder, modified time. Modified time comes from the
   file's metadata (native only; empty on wasm or when the file is missing) and is formatted as
   "Today, 14:32", "Yesterday", weekday within 7 days, else a date. A missing file is shown dimmed
   with "File not found" in the folder column.
6. Row actions: hover shows a pin toggle and a `⋯` button. `⋯` and right-click open a context
   menu: Open, Pin/Unpin, Show in Finder (Explorer / file manager; hidden when unavailable), Copy
   path, Remove from list.
7. Pins persist: `UiState.pinned: Vec<String>` (serde default, so old prefs load). Pinned
   documents also stay in Recent. Pinned list order = pin order, newest first. Removing from list
   removes from Recent only; unpin is separate.
8. Empty states: Recent empty → a small drawn illustration (page + sparkle, original) with
   "New document" and "Open file…" buttons. Pinned empty → "Pin documents to keep them here."
   Search with no hits → "Nothing matches your search."
9. Drop hint: while files are hovered over the window and Home is showing, draw an accent dashed
   outline with "Drop to open". Opening dropped files already works (lib.rs `dropped_files`).

Out (later): thumbnails of recent documents, the new icon set (sub-project 2), cloud/shared lists.

## Architecture

- New module `crates/ui-egui/src/home.rs` holds the Home page (`pub(crate) fn page(app, ui)`),
  the document list, the search filter and the time formatter. `backstage.rs` keeps navigation,
  the other pages and `template_tile` (made `pub(crate)` and given the hover-lift animation, so the
  New page benefits too). `backstage::home_page` delegates to `home::page`.
- Pure helpers, unit-tested without a UI:
  - `filter_docs(query, &[String]) -> Vec<String>`
  - `friendly_time(modified_local, now_local) -> String`, both local wall-clock times
    (`YYYY-MM-DD HH:MM` parts) so it is pure; the caller converts file mtimes with the same local
    clock source the greeting uses (`cmd::now_iso` and its helpers).
  - `kind_of(path) -> DocKind` (extension → icon tint).
  - `toggle_pin(&mut UiState, path)` and `remove_recent(&mut UiState, path)`.
- "Show in Finder" is a new optional service: `Services.reveal: Option<Box<dyn Fn(&str)>>`,
  set by `apps/wordcraft` (macOS `open -R`, Windows `explorer /select,`, Linux/BSD open the parent
  folder with the `open` crate already in that app). The UI hides the item when `None` (web,
  tests). Keeps L6 free of process spawning and wasm intact.
- Copy path uses `ctx.copy_text`.
- No new engine commands: these are UI-only behaviours on the Backstage, like the existing list.

## Never-crash rules

No `unwrap`/`expect`/indexing on paths or metadata; metadata errors → "missing" state; time maths
saturating; search on `to_lowercase` strings only (no byte slicing).

## Testing

- Unit tests in `home.rs` for every pure helper (filtering, pin toggling/persistence round-trip
  through `prefs()`/`apply_prefs`, time formatting edges: today/yesterday/week/older/future).
- egui_kittest tests in the style of `backstage.rs` tests: Home shows Recent and Pinned tabs,
  pinning moves a document to Pinned, context menu "Remove from list" removes it, search filters
  both templates and documents. `home_and_new_show_different_sections` is updated on purpose:
  Home now shows all five templates (its "Report is absent from Home" assertion goes), and the
  rest (More templates → New, New has no Recent list) still holds.
- `ui_shot` before/after screenshots of Home (empty and with recents) compared by eye.
- `cargo xtask ci` green before each commit.
