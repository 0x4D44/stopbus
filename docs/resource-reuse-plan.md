# Legacy Resource Reuse Plan

_Last updated: 20 September 2025_

## Goals
- Reuse the original `STOPBUS.RES` so the modern Rust binary renders the same card bitmaps, icons, dialogs, and accelerator tables as the 16-bit release.
- Preserve the historical binary for archival purposes while enabling gradual migration to editable `.rc`/`.bmp` sources.

## Runtime Loading Strategy
1. Keep the shipped `STOPBUS.RES` in the repository root (or move to `assets/legacy/` once the build pipeline copies it automatically).
2. During startup the Rust UI calls `LoadLibraryExW(..., LOAD_LIBRARY_AS_DATAFILE)` to load the `.res` as a module (see `stopbus-ui/src/main.rs`).
3. Store the resulting `HMODULE` inside `WindowState` so later rendering code can issue `LoadBitmapW`, `FindResourceW`, or `LoadAcceleratorsW` against the legacy module.
4. Release the module with `FreeLibrary` when the window is destroyed (handled in `Drop` for `WindowState`).

## Next Steps
- Add helper wrappers that fetch card bitmaps by numeric ID, mirroring the Pascal lookups (`CARD01` ? resource ordinal 1, etc.).
- Introduce a build script to copy `STOPBUS.RES` into the Cargo output directory so the executable can locate it when run outside the repository root.
- Gradually replace portions of the `.res` with source-controlled `.rc` definitions once equivalent assets are extracted (e.g., About dialog, icons).

## Tooling Considerations
- Use `Resource Hacker` or `wrestool` to enumerate resource IDs for cards and UI elements; document the mapping in a structured file under `docs/`.
- When alternative formats (PNG, SVG) are created, retain the original bitmap IDs to avoid rewriting large sections of code.
- If we later embed resources at link time, wire up `windows-resource` to compile an `.rc` shell that `#include`s the legacy `.res` until the full migration completes.
