# Repository Guidelines

## Project Structure & Module Organization
Legacy Pascal sources (`STOPBUS.PAS`, `CTL3D.PAS`, backups) and resource scripts (`ABOUT.RC`, `STOPBUS.RES`) remain at the root alongside release artifacts. WinHelp material lives in `HELP/`. Modernization work sits in `crates/`: `stopbus-core` (Rust library for game logic) and `stopbus-ui` (windows-rs shell). Documentation now resides in `docs/`, including architecture notes and help-inventory reports, while the modernization roadmap is tracked in `2025.09.20 - Migration & updating plan.md`. Use `RELEASE/` to stage binaries destined for distribution.

## Build, Test, and Development Commands
Legacy: run `rc /r ABOUT.RC` or `powershell -ExecutionPolicy Bypass -File .\RESOURCE.BAT`, then rebuild with Turbo Pascal (`tpcw /B STOPBUS.PAS`) and compile help via `hc31 HELP\STOPBUS.HPJ`. Modern: from the repository root run `cargo check` (fast validation) or `cargo build --release` to produce the Rust executable under `target\release\stopbus.exe`. When packaging, verify parity with `fc /b STOPBUS.TXT RELEASE\STOPBUS.TXT` and `fc /b target\release\stopbus.exe RELEASE\STOPBUS.EXE` (or the legacy binary being replaced). Use `Compress-Archive RELEASE\* -DestinationPath RELEASE\STOPBUS.zip` to bundle deliverables.

## Coding Style & Naming Conventions
Pascal modules keep two-space indentation, uppercase identifiers, and 8.3 filenames; retain CRLF endings and `(* ... *)` comments. Rust crates follow Rustfmt defaults (4-space indent, snake_case modules). Group shared Rust logic under `stopbus-core` and keep Win32 bindings in `stopbus-ui`. For resources and docs, prefer ASCII markdown with 72–80 character soft wrap.

## Testing Guidelines
For legacy parity, validate `STOPBUS.EXE` on a Windows 3.1 or Windows 95 VM. The Rust build should at minimum pass `cargo test` (once tests exist) and a manual smoke test on Windows 10/11: launch, trigger menu commands, and ensure the bootstrap MessageBox appears. Use `findstr /R /C:"[^ -~]"` across Pascal and documentation files to catch non-ASCII drift, and capture `certutil -hashfile` outputs for executables, resources, and help files in PR notes.

## Commit & Pull Request Guidelines
Write imperative 50-character subjects (e.g., "Bootstrap Rust workspace"). Reference issues with `Fixes #n` when applicable, and list regenerated artifacts (Pascal binaries, help build, Rust target). PR descriptions should summarize scope, manual verification (`cargo check`, legacy EXE run, hash logs), and note changes under `RELEASE/`. Include links to supporting docs (`docs/stopbus-architecture-notes.md`, etc.) when relevant.

## Preservation Notes
Retain original timestamps and hashes for all legacy binaries; document any regenerated files in PRs and within `RELEASE/README` if touched. Keep backups (`STOPBUS.BAK`, `STOPBUS.~RE`) untouched for historical comparison. When converting help assets or resources, store raw exports under `assets/` (to be created) alongside the original `HELP/BMP.ZIP`, and record tooling provenance for each conversion step.



