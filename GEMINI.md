# Stop the Bus (GEMINI Context)

## Project Overview
This project is a preservation and modernization effort for "Stop the Bus," a Windows card game originally released in 1993. It involves two distinct components:
1.  **Legacy Archive:** The original Turbo Pascal for Windows 1.5 source code (`STOPBUS.PAS`), resources, and help files.
2.  **Modern Rewrite:** A Rust-based implementation targeting modern Windows using the `windows` crate, aiming for feature parity.

The project is currently in the **modernization phase**, following a structured migration plan.

## Architecture & Structure

### Key Directories
*   **`.` (Root):** Legacy Pascal sources (`STOPBUS.PAS`), build scripts (`RESOURCE.BAT`), and documentation.
*   **`crates/`:** The modern Rust workspace.
    *   `stopbus-core`: Platform-agnostic game logic (deck, scoring, player state).
    *   `stopbus-ui`: The Win32 user interface implementation.
*   **`assets/`:** Extracted and cleaned assets (images, icons) for the modern build.
*   **`docs/`:** Project documentation, including architecture notes and the migration roadmap.
*   **`HELP/` & `help_bmp/`:** Original WinHelp source files and bitmaps.

### Technology Stack
*   **Language:** Rust (2021 Edition)
*   **Windows API:** `windows` crate (Win32 UI)
*   **Legacy:** Turbo Pascal for Windows (TPW 1.5), WinHelp

## Building and Running

### Modern Rust Build
The modern version is a standard Cargo workspace.

**Build:**
```bash
cargo build --release
```

**Run:**
```bash
target\release\stopbus.exe
```

**Test:**
```bash
cargo test
```

### Legacy Build
Building the legacy version requires an environment with Turbo Pascal for Windows 1.5. This is primarily for archival and comparison purposes.

## Development Conventions

### Migration Roadmap
Work follows the plan outlined in `2025.09.20 - Migration & updating plan.md`. Key phases include:
1.  **Reverse Engineering:** Understanding `STOPBUS.PAS` (Completed/In-progress).
2.  **Asset Extraction:** Converting resources to modern formats.
3.  **Core Logic:** Implementing `stopbus-core` with unit tests.
4.  **UI Implementation:** Recreating the Win32 interface in `stopbus-ui`.

### Coding Standards
*   **Rust:** Follow standard Rust idioms and `clippy` suggestions.
*   **Win32:** The UI code interacts directly with the Windows API. Ensure unsafe blocks are carefully managed and documented.
*   **Parity:** The goal is to replicate the original game's behavior and look-and-feel.

### Documentation
*   Refer to `docs/stopbus-architecture-notes.md` for details on the legacy code structure and message flow.
*   `AGENTS.md` contains specific instructions for AI agents and contributors.
