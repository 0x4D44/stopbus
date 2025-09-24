# Stop the Bus (Preservation + Modernization)

This repository preserves a 1993 Windows card game and hosts a Rust/windows-rs rewrite.

## Repository Layout
- Legacy Pascal sources and resources: `STOPBUS.PAS`, `CTL3D.PAS`, `ABOUT.RC`, `RESOURCE.BAT`.
- Original release artifacts: `STOPBUS.EXE`, `STOPBUS.HLP`, documentation under `STOPBUS.TXT`, backups, and the `RELEASE/` staging folder.
- WinHelp authoring files: `HELP/STOPBUS.HPJ`, `STOPBUS.RTF`, `BMP.ZIP`, diagnostic logs.
- Modern Rust workspace: `crates/stopbus-core` (core logic) and `crates/stopbus-ui` (Win32 shell using the `windows` crate).
- Documentation: `docs/stopbus-architecture-notes.md`, `docs/help-assets-inventory.md`, and the modernization roadmap `2025.09.20 - Migration & updating plan.md`.

## Getting Started (Modern Build)
1. Install a recent Rust toolchain (stable channel) and ensure the MSVC build target is available.
2. From the repository root run `cargo check` to validate dependencies.
3. Run `cargo build --release` to produce `target\release\stopbus.exe`.
4. (Optional) Launch the legacy Pascal executable under a Windows 3.1/95 VM for behavioral comparison.

## Documentation & Next Steps
- Review the migration roadmap (`2025.09.20 - Migration & updating plan.md`) for the phased plan toward parity.
- Consult `docs/stopbus-architecture-notes.md` for the legacy application's structure and message flow.
- See `docs/help-assets-inventory.md` for WinHelp asset handling and conversion guidance.
- Contributor practices and testing expectations are summarized in `AGENTS.md`.

## Licensing & Attribution
The modernization code is published under MIT (see `Cargo.toml`). 



