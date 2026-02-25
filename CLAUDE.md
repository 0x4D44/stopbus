# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a preservation and modernization project for "Stop the Bus," a 1993 Windows card game originally written in Turbo Pascal. The repository contains:
- **Legacy codebase**: Original Turbo Pascal for Windows sources (`STOPBUS.PAS`, `CTL3D.PAS`), resources (`ABOUT.RC`, `STOPBUS.RES`), and WinHelp files
- **Modern Rust rewrite**: A feature-parity reimplementation using Rust and windows-rs targeting modern Windows

The goal is to preserve the original as a documented artifact while delivering a fully functional Rust version with identical gameplay.

## Build Commands

### Modern Rust Build (Primary Development)

```bash
# Fast validation of all crates
cargo check

# Build release executable (produces target\release\stopbus.exe)
cargo build --release

# Run tests across workspace
cargo test

# Run tests for specific crate
cargo test -p stopbus-core

# Run specific test
cargo test <test_name>

# Check formatting
cargo fmt --check

# Apply formatting
cargo fmt

# Run clippy lints
cargo clippy -- -D warnings
```

### Legacy Pascal Build (Reference Only)

The legacy build requires Turbo Pascal for Windows 1.5 and is used only for verification:

```bash
# Compile resources
rc /r ABOUT.RC
# or
powershell -ExecutionPolicy Bypass -File .\RESOURCE.BAT

# Compile Pascal (requires TPW 1.5 in Windows VM)
tpcw /B STOPBUS.PAS

# Build help file
hc31 HELP\STOPBUS.HPJ
```

## Architecture

### Cargo Workspace Structure

This is a Cargo workspace with two crates:

- **`crates/stopbus-core`**: Pure Rust library containing game logic, card deck management, scoring rules, player state, and turn sequencing. No UI dependencies. All game logic should be unit tested here.

- **`crates/stopbus-ui`**: Windows executable using `windows-rs` (windows crate v0.58+) for Win32 bindings. Contains the main window, message loop, dialogs, rendering, and resource embedding. Depends on `stopbus-core`.

### Key Design Principles

1. **Separation of Concerns**: Game logic lives in `stopbus-core`, UI/Win32 code lives in `stopbus-ui`
2. **Message-Driven UI**: The Win32 UI follows Windows message-driven architecture via `WndProc` pattern
3. **Resource Embedding**: Resources (icons, bitmaps, dialogs) are embedded at build time via `embed-resource` in `stopbus-ui/build.rs`
4. **Legacy Parity**: The Rust implementation must match the original Pascal behavior exactly

### Original Pascal Architecture (Reference)

The legacy code (`STOPBUS.PAS`) uses Turbo Vision for Windows with:
- `TCardApp` (application object) hosting `TMainWindow`
- Global arrays for game state (`PackCard[1..52]`, `Player[1..4,1..3]`, `RoundScore[1..4]`)
- Message handlers as virtual methods (`CM_*`, `WM_*` commands)
- Dialogs: `TOptions` (cheat/settings), `TAboutBox`, cheat windows (`TCheatCards`, `TCheatScores`, `TCheatStack`)
- Core game methods: `Shuffle`, `Deal`, `GameControl`, `Scores`, `StopTheBus`

In the Rust port, these globals are encapsulated in structs, and message handlers are unified in the `WndProc`.

## Testing Strategy

### Unit Testing
- All core game logic in `stopbus-core` must have comprehensive unit tests
- Test scoring rules, deck shuffling, player state transitions, and edge cases
- Run: `cargo test -p stopbus-core`

### Integration Testing
- Manual smoke testing on Windows 10/11:
  1. Launch `target\release\stopbus.exe`
  2. Verify menu commands work
  3. Play through a full game
  4. Test dialog interactions (Options, About)
  5. Verify card rendering and interactions

### Legacy Parity Testing
- Compare Rust behavior against original `STOPBUS.EXE` running in Windows 3.1/95 VM
- Validate scoring matches Pascal implementation
- Use `certutil -hashfile` to document binary hashes

## Development Workflow

### Making Changes

1. **Core Logic Changes**: Edit `crates/stopbus-core/src/lib.rs`, write tests first (TDD)
2. **UI Changes**: Edit `crates/stopbus-ui/src/main.rs`
3. **Resources**: Update `crates/stopbus-ui/res/` files and ensure `build.rs` embeds them
4. Always run `cargo check` and `cargo test` before committing
5. Ensure `cargo fmt` and `cargo clippy` pass cleanly

### Common Tasks

- **Add new game feature**: Start in `stopbus-core` with tests, then wire to UI in `stopbus-ui`
- **Fix scoring bug**: Write failing test in `stopbus-core`, fix logic, verify test passes
- **Update dialog**: Modify resource files in `stopbus-ui/res/`, rebuild
- **Debug Win32 issues**: Reference `docs/stopbus-architecture-notes.md` for legacy behavior

## Important Files & Documentation

- **`2025.09.20 - Migration & updating plan.md`**: Phased modernization roadmap (Phases A-G)
- **`docs/stopbus-architecture-notes.md`**: Detailed analysis of legacy Pascal code structure and message flows
- **`docs/resource-id-map.md`**: Mapping of resource IDs from Pascal to Rust
- **`AGENTS.md`**: Repository guidelines for preservation, testing, and commit practices

## Coding Standards

### Rust Code
- Follow Rustfmt defaults (4-space indent, snake_case)
- All code must compile with zero warnings (`cargo clippy -- -D warnings`)
- Avoid `#[allow(dead_code)]` and `unsafe` unless absolutely necessary
- Group shared logic in `stopbus-core`, Win32 bindings in `stopbus-ui`

### Pascal Code (Preservation)
- Keep legacy code unchanged unless explicitly required
- Maintain CRLF line endings, two-space indent, uppercase identifiers
- Do not modify backups (`STOPBUS.BAK`, `STOPBUS.~RE`)

## Resource and Asset Management

- **Card Bitmaps**: Numbered `CARD01`-`CARD52` plus `CardBack`, embedded via resource compiler
- **Icons**: `MD.ICO` and `ICON_1`/`ICON_2` referenced in resources
- **Help Files**: Legacy WinHelp (`.HLP`) being migrated to HTML/Markdown (see `docs/help-assets-inventory.md`)
- **Dialogs**: Defined in `.rc` files, embedded at build time

## Migration Status

The project is currently in **Phase C-D** (Rust bootstrap and core logic port). Key workstreams:
- ✅ Phase A: Reverse engineering documented
- ✅ Phase B: Asset extraction in progress
- ✅ Phase C: Rust workspace bootstrapped
- 🔄 Phase D: Core logic port ongoing
- ⏳ Phase E: UI layer (next major phase)
- ⏳ Phase F-G: Help refresh and packaging

Consult `2025.09.20 - Migration & updating plan.md` for full phase breakdown and timeline.

## Platform & Tooling

- **Target OS**: Windows 10/11 (64-bit)
- **Rust Version**: Stable channel (currently 1.90.0)
- **MSVC Toolchain**: Required for windows-rs
- **Legacy Testing**: Windows 3.1/95 VM with Turbo Pascal for Windows 1.5

## Key Dependencies

- `windows = "0.58"` with features: `Win32_Foundation`, `Win32_UI_WindowsAndMessaging`, `Win32_Graphics_Gdi`, `Win32_System_LibraryLoader`, `Win32_System_Registry`, `Win32_UI_Input`
- `rand = "0.8"` with `std` feature for deck shuffling
- `embed-resource = "3.0"` for embedding Windows resources at build time

## Preservation Notes

- Keep original timestamps and hashes for all legacy binaries documented
- Never modify backups or historical files
- Store converted assets in `assets/` alongside originals
- Document provenance of all resource conversions in PR descriptions
- Reference legacy behavior explicitly when implementing Rust equivalents
