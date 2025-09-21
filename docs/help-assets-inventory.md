# WinHelp Asset Inventory and Conversion Approach

_Last reviewed: 20 September 2025_

## Asset Overview
- **HELP/STOPBUS.HPJ** — WinHelp 3.1 project file targeting `STOPBUS.RTF`; defines `options` topic map (ID 1) and single window profile named `main` with gray background (RGB 192,192,192).
- **HELP/STOPBUS.RTF** — Rich Text source for all topics; uses WinHelp footnotes for keywords, browse sequencing, and the `main_contents` table.
- **HELP/STOPBUS.HLP** — Compiled WinHelp binary generated from the project; legacy runtime requirement.
- **HELP/STOPBUS.ERR** — Build log referencing issues encountered by `hc31`.
- **HELP/STOPBUS.GID** — Generated index cache used by WinHelp during runtime; not source-controlled in the Rust port.
- **HELP/BMP.ZIP** — Embedded graphics bundle containing:
  - `CARDWIND.BMP`, `SCORWIND.BMP`, `STACWIND.BMP` (24-bit screen captures used in topics).
  - `MAINICON.BMP` (icon preview).
  - `MAINWIN.SHG`, `OPTNWIN.SHG` (segmented hypergraphics for hotspot-driven callouts).

## Proposed Modernization Path
1. **Source Control Extraction**
   - Unzip `HELP/BMP.ZIP` into `assets/help/raw/` while retaining the original archive for provenance.
   - Convert `.SHG` files to static PNG using `winhelpcgi` or `shg2bmp` utilities inside a Windows 98 VM (document the process and retain converted assets alongside originals).
2. **RTF to Markdown/HTML**
   - Use `pandoc` (`pandoc HELP/STOPBUS.RTF -f rtf -t gfm -o docs/help/stopbus.md`) to create an editable Markdown source.
   - Preserve WinHelp-specific metadata by exporting footnotes to a YAML front matter block or custom comments, enabling reconstruction of context-sensitive help IDs.
3. **Context Mapping**
   - Record the `MAP` section (`options = 1`) in a JSON/YAML manifest (`docs/help/map.json`) so the Rust UI can open specific sections.
4. **In-App Delivery**
   - Embed converted Markdown/HTML in the Rust application using a simple WebView2-based dialog or external browser launch until an in-app renderer is chosen.
5. **Build Integration**
   - Add a Cargo build script to copy converted assets into the output directory; include checksum verification to ensure regenerated help stays in sync.

## Outstanding Questions
- Do we retain segmented hypergraphics interactivity, or replace them with annotated screenshots in the modern help?
- Should the Rust build offer an offline HTML viewer or rely on the system browser?
- Where should the legacy `.HLP` reside (e.g., `RELEASE/legacy/`) for archival purposes?
