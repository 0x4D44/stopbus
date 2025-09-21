# Embedded Resource Build Notes

- `stopbus-ui/build.rs` now checks for `STOPBUS.RES` when building on Windows. If found, it copies the resource into `OUT_DIR` so the UI crate can embed the raw bytes via `include_bytes!` and toggle the `EMBED_STOPBUS_RES` environment flag, yielding a single `stopbus.exe` with the legacy bitmaps and dialogs bundled.
- The build script also emits `cargo:rerun-if-changed` for the resource, ensuring incremental builds re-link when the `.RES` changes. If the file is missing the build still succeeds but prints a warning; at runtime the loader falls back to searching beside the executable.
- Keep `STOPBUS.RES` in the repo root (tracked) so local builds embed it automatically. CI should either copy the file into place before building or treat the warning as an error.
- To verify embedding, inspect the release binary with `Resource Hacker` or `dumpbin /headers` and check that BITMAP entries 1–54 exist.


