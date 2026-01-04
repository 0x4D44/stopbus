# Stop the Bus

**A preservation and modernization project for the classic 1993 Windows card game.**

This repository serves as both a historical archive and a modern revitalization of "Stop the Bus," originally written in Turbo Pascal for Windows. It hosts two distinct implementations:

1.  **Legacy Archive:** The original source code (`STOPBUS.PAS`), assets, and help files from the 1993 release.
2.  **Modern Rewrite:** A native, pixel-perfect Rust implementation targeting modern Windows (10/11) using the Win32 API.

---

## 🚀 Getting Started

### Prerequisites
*   **Operating System:** Windows 10 or 11 (required for the modern UI).
*   **Toolchain:** Rust (Stable) with the `x86_64-pc-windows-msvc` target.

### Building & Running
Clone the repository and run the following commands in the root directory:

```bash
# Build the project in release mode
cargo build --release

# Run the game
cargo run --bin stopbus-ui
```

The compiled executable will be located at `target/release/stopbus-ui.exe`.

---

## 🏗️ Project Architecture

The modern implementation is organized as a Cargo workspace with a clear separation of concerns:

### `crates/stopbus-core`
*   **Role:** The platform-agnostic game engine.
*   **Responsibility:** Handles all game logic, including deck management, scoring rules (31 points, pairs, distinct suits), AI decision-making, and state machine transitions.
*   **Quality Standard:**
    *   **Test Coverage:** **>98% line coverage** (verified via `llvm-cov`).
    *   **Verification:** rigorously tested against complex edge cases like deck overflows, "Stop the Bus" tie-breaking, simultaneous player deaths, and lone survivor scenarios.

### `crates/stopbus-ui`
*   **Role:** The Windows presentation layer.
*   **Tech Stack:** Built using the `windows` crate (windows-rs) for direct, unsafe access to the Win32 API.
*   **Goal:** Replicates the visual style and behavior of the original 1993 interface (GDI graphics, menus, dialogs) while running natively on modern hardware without emulation.

### `assets/` & `HELP/`
*   **Legacy Resources:** Contains extracted bitmaps (`.bmp`), icons (`.ico`), cursors, and the original WinHelp source files (`.hlp`, `.rtf`).
*   **Preservation:** The root directory retains the original `STOPBUS.PAS` source code for historical reference and behavioral comparison.

---

## 🛠️ Development Workflow

We maintain strict standards for code quality and correctness, especially for the core game logic.

### Testing
Run the comprehensive test suite to verify game rules and logic:

```bash
# Run all unit tests
cargo test

# Run tests with detailed output (useful for debugging)
cargo test -- --nocapture
```

### Code Quality
Ensure your changes meet the project's style and linting guidelines before submitting:

```bash
# Check for compilation errors
cargo check

# Verify code formatting
cargo fmt --all -- --check

# Run linter (configured to treat warnings as errors)
cargo clippy --all-targets --all-features -- -D warnings
```

---

## 📜 History & Roadmap

The original "Stop the Bus" was a shareware game created in 1993 using Borland Turbo Pascal for Windows 1.5. This project aims to preserve that piece of software history while providing a robust, maintainable codebase for future improvements.

**Current Status:**
- [x] **Core Logic:** Fully ported to Rust with >98% test coverage.
- [ ] **UI Implementation:** In progress (Win32 scaffolding and basic windowing in place).
- [ ] **Asset Integration:** Loading original bitmaps and resources.

For detailed archival notes and the migration plan, please consult:
- `2025.09.20 - Migration & updating plan.md`
- `docs/stopbus-architecture-notes.md`

---

## ⚖️ License

The modern Rust implementation is licensed under the **MIT License**.
The legacy assets and original Pascal source code are included for preservation and educational purposes.
