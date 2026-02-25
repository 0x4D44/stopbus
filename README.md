# Stop the Bus (Preservation + Modernization)

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.90%2B-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Windows%2010%2F11-blue.svg)](https://www.microsoft.com/windows)

This repository preserves a 1993 Windows card game originally written in Turbo Pascal and hosts a modern Rust/windows-rs rewrite with feature parity.

**Original Author:** Martin (M.G.) Davidson, Hertford College, Oxford University
**Original Release:** v1.11 (May 9, 1994)
**Modern Version:** v2.0 (Rust modernization)
**Migration Status:** Phase D - 60% Complete

---

## 📖 Quick Links

- **[Comprehensive Architecture Documentation](2025.11.06%20-%20COMPREHENSIVE-ARCHITECTURE.md)** - Complete system architecture, data flows, and technical details
- **[Migration Roadmap](2025.09.20%20-%20Migration%20&%20updating%20plan.md)** - Phased modernization plan (Phases A-G)
- **[Contributor Guidelines](AGENTS.md)** - Repository practices and testing expectations
- **[Claude Code Integration](CLAUDE.md)** - Guide for AI-assisted development

---

## 🎮 About the Game

**Stop the Bus** is a 4-player card game (1 human + 3 AI) where players compete to avoid losing all their lives.

**Game Rules:**
- Each player starts with **3 lives**
- Players are dealt **3 cards** per round
- Goal: Build the highest **suit-matched score** (Ace=11, face cards=10)
- **Stop the Bus:** Score of 31 immediately ends the round
- **Sticking:** Players can "stick" at score >25 to force round end
- **Loser:** Player(s) with lowest score lose a life
- **Winner:** Last player with lives remaining

---

## 🚀 Quick Start (Modern Rust Build)

### Prerequisites

- **Rust:** 1.90.0 or later (stable channel)
- **MSVC Toolchain:** Required for windows-rs (install via Visual Studio Build Tools)
- **Platform:** Windows 10/11 (64-bit)

### Build Commands

```bash
# Validate all crates
cargo check

# Build debug version
cargo build

# Build optimized release version
cargo build --release
# Output: target\release\stopbus.exe

# Run all tests
cargo test

# Run tests for specific crate
cargo test -p stopbus-core

# Format code
cargo fmt

# Lint with Clippy
cargo clippy -- -D warnings
```

### Running the Game

```bash
# Run debug build
cargo run

# Or directly execute release build
.\target\release\stopbus.exe
```

---

## 📁 Repository Structure

```
stopbus/
├── 2025.11.06 - COMPREHENSIVE-ARCHITECTURE.md  # 📘 Complete technical documentation
├── 2025.09.20 - Migration & updating plan.md   # Phased modernization roadmap
├── AGENTS.md                                    # Contributor guidelines
├── CLAUDE.md                                    # Claude Code integration guide
├── README.md                                    # This file
│
├── crates/                                      # Modern Rust workspace
│   ├── stopbus-core/                            # Game logic library (1,072 lines)
│   │   ├── Cargo.toml
│   │   └── src/lib.rs                           # Pure Rust game logic
│   └── stopbus-ui/                              # Win32 executable (2,571 lines)
│       ├── Cargo.toml
│       ├── build.rs                             # Resource embedding
│       ├── resources/                           # Embedded resources
│       │   ├── stopbus.rc                       # Resource script
│       │   └── cards/*.bmp                      # 54 card bitmaps
│       └── src/main.rs                          # Win32 UI implementation
│
├── docs/                                        # Technical documentation
│   ├── stopbus-architecture-notes.md            # Legacy Pascal analysis
│   ├── resource-id-map.md                       # Resource ID reference
│   ├── help-assets-inventory.md                 # WinHelp migration planning
│   ├── resource-embedding-plan.md               # Build strategy
│   └── card-bitmap-normalization-log.md         # Asset conversion log
│
├── wrk_docs/                                    # Working documentation
│   ├── README.md                                # Documentation index
│   ├── 2025.11.06 - stopbus-core-architecture.md
│   ├── 2025.11.06 - stopbus-ui-architecture.md
│   ├── 2025.11.06 - legacy-pascal-analysis.md
│   ├── 2025.11.06 - resource-build-pipeline.md
│   └── 2025.11.06 - project-structure-overview.md
│
├── assets/                                      # Extracted/converted resources
│   ├── cards-os2/                               # Original OS/2 format BMPs
│   └── original-icons/                          # Extracted icons
│
├── tools/                                       # Build automation scripts
│   ├── extract_cards.py                         # OS/2 → Windows V3 converter
│   └── update_about_text.py                     # Resource text manipulation
│
├── installers/
│   └── StopTheBus.iss                           # Inno Setup installer script
│
├── HELP/                                        # Legacy WinHelp system
│   ├── STOPBUS.HLP                              # Compiled help (deprecated)
│   ├── STOPBUS.HPJ                              # Help project definition
│   ├── STOPBUS.RTF                              # Help source content
│   └── BMP.ZIP                                  # Help graphics
│
├── RELEASE/                                     # Distribution staging
│   ├── STOPBUS.ZIP                              # Original distribution
│   └── STOPBUS.TXT                              # Release notes
│
├── STOPBUS.PAS                                  # 🏛️ Original Pascal source (2,528 lines)
├── CTL3D.PAS                                    # 3D control library wrapper
├── ABOUT.RC                                     # About dialog resources
├── STOPBUS.RES                                  # Compiled resources (100KB)
├── STOPBUS.EXE                                  # Original executable
├── STOPBUS.TXT                                  # Shareware documentation
├── MD.ICO                                       # Application icon
└── Cargo.toml                                   # Rust workspace definition
```

---

## 🏗️ Architecture Overview

### Modern Rust Implementation

**Two-Crate Design:**

1. **stopbus-core** (Pure Logic Library)
   - No UI dependencies
   - Game state management
   - Card scoring algorithms (matches Pascal exactly)
   - AI turn logic (>25 stick, 2-stage swap)
   - 100-iteration Fisher-Yates shuffle (preserves original behavior)
   - Comprehensive unit tests with seeded RNG

2. **stopbus-ui** (Win32 Executable)
   - Direct Win32 APIs via windows-rs
   - Message-driven architecture (WndProc)
   - GDI rendering (BitBlt for cards, TextOut for scores)
   - Registry persistence (settings, window positions)
   - Cheat windows (show AI hands, stack preview, scores)
   - Dual resource loading (legacy STOPBUS.RES + embedded)

**Key Technologies:**
- `windows-rs 0.58` for Win32 bindings
- `rand 0.8` for deterministic shuffling
- `embed-resource 3.0` for compile-time resource embedding

### Legacy Preservation

**Original 1993 Implementation:**
- **Language:** Turbo Pascal for Windows 1.5
- **Platform:** Windows 3.1 with CTL3D.DLL
- **Architecture:** Object-oriented (TApplication → TMainWindow)
- **Resources:** OS/2 format bitmaps, WinHelp system
- **Distribution:** Shareware (£5 UK / $10 USD)

All original files preserved in repository root for historical reference and comparison.

---

## 📊 Migration Progress

```
┌────────────────────────────────────────────────────────────┐
│ Phase A: Reverse Engineering          [████████████] 100% │
│ Phase B: Asset Extraction              [████████████] 100% │
│ Phase C: Rust Bootstrap                [████████████] 100% │
│ Phase D: Core Logic Port               [████████░░░░]  60% │
│ Phase E: UI Interaction                [░░░░░░░░░░░░]   0% │
│ Phase F: Help System Modernization     [░░░░░░░░░░░░]   0% │
│ Phase G: Packaging & QA                [░░░░░░░░░░░░]   0% │
└────────────────────────────────────────────────────────────┘
```

**Current Focus:** Completing Phase D (core logic port)

**Recent Achievements:**
- ✅ Complete game state machine implemented
- ✅ AI logic matches Pascal AutoPlay exactly
- ✅ Scoring algorithm verified (100% parity)
- ✅ All card bitmaps converted to Windows V3 format
- ✅ Win32 UI with working message loop and rendering

**Next Milestones:**
- [ ] Expand unit test coverage to 90%+
- [ ] Complete turn sequencing and life tracking
- [ ] Wire all UI interactions (drag/drop, buttons)
- [ ] Implement modern help system (Phase F)

See **[2025.09.20 - Migration & updating plan.md](2025.09.20%20-%20Migration%20&%20updating%20plan.md)** for detailed phase breakdown.

---

## 🧪 Testing

### Unit Tests (stopbus-core)

```bash
# Run all core logic tests
cargo test -p stopbus-core

# Run with output
cargo test -p stopbus-core -- --nocapture

# Run specific test
cargo test test_name -p stopbus-core
```

**Coverage:**
- ✅ Card rank/suit extraction
- ✅ Point value calculations
- ✅ Hand scoring (suit-matched combinations)
- ✅ Stop the Bus detection (score == 31)
- ✅ Deal respects lives (dead players get no cards)

### Integration Testing (Manual)

1. Build release: `cargo build --release`
2. Run: `.\target\release\stopbus.exe`
3. Verify:
   - Card rendering (stack, hand)
   - Score calculations
   - AI turns (watch cheat windows)
   - Life tracking
   - Game over conditions

### Legacy Parity Testing

Compare behavior with original `STOPBUS.EXE` in Windows 3.1/95 VM:
- Scoring calculations match exactly
- AI decision-making identical (>25 stick, 2-stage swap)
- Shuffle randomness equivalent (100 iterations)

---

## 🛠️ Development Workflow

### Making Changes

**Core Logic (stopbus-core):**
1. Write failing test (TDD)
2. Implement minimal fix in `crates/stopbus-core/src/lib.rs`
3. Run `cargo test -p stopbus-core`
4. Refactor while keeping tests green
5. Run `cargo fmt` and `cargo clippy`

**UI Changes (stopbus-ui):**
1. Edit `crates/stopbus-ui/src/main.rs`
2. Update resources in `crates/stopbus-ui/resources/` if needed
3. Build and test manually: `cargo run`
4. Verify rendering and interactions

**Resource Changes:**
1. Update files in `crates/stopbus-ui/resources/`
2. Modify `stopbus.rc` for dialogs/menus
3. Rebuild (build.rs embeds automatically)
4. Verify with Resource Hacker or dumpbin

### Commit Guidelines

```
<verb> <concise description>

- Detail 1
- Detail 2

Tested: cargo check, cargo test
Artifacts: target/release/stopbus.exe (SHA256: ...)

Fixes #<issue>
```

---

## 📚 Documentation

### Architecture Documentation

**[2025.11.06 - COMPREHENSIVE-ARCHITECTURE.md](2025.11.06%20-%20COMPREHENSIVE-ARCHITECTURE.md)** (Root)
- **Executive summary** of entire project
- **System architecture** with ASCII diagrams
- **Data flow** diagrams (initialization, turns, automation, resources)
- **Component integration** points
- **Technology stack** breakdown
- **Migration status** and roadmap

### Detailed Component Docs (wrk_docs/)

- **stopbus-core-architecture.md** - Game logic library deep dive
- **stopbus-ui-architecture.md** - Win32 UI implementation details
- **legacy-pascal-analysis.md** - Original Pascal code analysis
- **resource-build-pipeline.md** - Build system and resource flow
- **project-structure-overview.md** - Repository organization (40KB!)

### Technical Docs (docs/)

- **stopbus-architecture-notes.md** - Legacy Pascal reverse engineering
- **resource-id-map.md** - Resource ID reference (1-54)
- **help-assets-inventory.md** - WinHelp migration planning
- **resource-embedding-plan.md** - Build-time embedding strategy
- **card-bitmap-normalization-log.md** - OS/2 → Windows V3 conversion log

---

## 🎯 Project Goals

### Preservation
- ✅ Complete original Pascal codebase preserved
- ✅ All original resources archived (bitmaps, icons, help files)
- ✅ Build artifacts documented with hashes
- ✅ Historical context captured in documentation

### Modernization
- ✅ Feature-parity Rust implementation (60% complete)
- ✅ Modern type safety and memory management
- ✅ Comprehensive documentation ecosystem
- ✅ Clean architectural separation (core vs. UI)
- ⏳ Modern help system (pending Phase F)
- ⏳ Automated build/test pipeline (pending Phase G)

### Fidelity
- ✅ Exact scoring algorithm preserved
- ✅ Identical AI behavior (>25 stick threshold, 2-stage swap)
- ✅ Same shuffle randomness (100 iterations)
- ✅ Original UI layout maintained (600×400, card positions)
- ✅ Card graphics pixel-perfect (via legacy STOPBUS.RES support)

---

## 🤝 Contributing

We welcome contributions! Please:

1. Read **[AGENTS.md](AGENTS.md)** for repository guidelines
2. Review **[2025.09.20 - Migration & updating plan.md](2025.09.20%20-%20Migration%20&%20updating%20plan.md)** for current priorities
3. Check **[2025.11.06 - COMPREHENSIVE-ARCHITECTURE.md](2025.11.06%20-%20COMPREHENSIVE-ARCHITECTURE.md)** for technical details
4. Follow TDD approach (write tests first)
5. Ensure `cargo fmt` and `cargo clippy` pass
6. Reference Pascal original for behavior verification

### Good First Issues

- [ ] Expand unit test coverage (stopbus-core)
- [ ] Improve error messages in UI
- [ ] Add DPI awareness for modern displays
- [ ] Convert WinHelp to HTML/Markdown
- [ ] Implement statistics tracking

---

## 📜 Licensing & Attribution

**Modern Code (Rust):** MIT License (see `Cargo.toml`)
**Original Game (1993):** © M.G. Davidson, published as shareware
**Card Graphics:** Microsoft standard playing card bitmaps
**Modernization:** Community-maintained (2025)

### Original Credits

```
Stop The Bus v1.11 (9th May 1994)
Written by: M. Davidson
            Hertford College
            Oxford University
            Oxford, England

Original distribution: Shareware (£5 UK / $10 US)
```

### Modernization Credits

- Rust port and architecture: Community contributors
- Documentation: Claude Code (Sonnet 4.5) assisted
- Testing: Community contributors
- Preservation: Repository maintainers

---

## 🔗 Additional Resources

- **Legacy Testing:** Use Windows 3.1/95 VM with Turbo Pascal for comparison
- **Resource Tools:** Resource Hacker (GUI), dumpbin (CLI)
- **Asset Conversion:** `tools/extract_cards.py` for BMP format conversion
- **Build Troubleshooting:** Check `build.rs` validates all card BMPs as Windows V3

---

## 📧 Support

- **Documentation Issues:** Check [wrk_docs/README.md](wrk_docs/README.md) for navigation
- **Build Problems:** Ensure MSVC toolchain installed, run `cargo clean && cargo build`
- **Game Behavior Questions:** Compare with original Pascal (`STOPBUS.PAS`) or legacy docs

---

**Project Status:** Phase D (Core Logic Port) - 60% Complete
**Last Updated:** 2025-11-06
**Next Milestone:** Complete Phase D unit tests and turn sequencing
