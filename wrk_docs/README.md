# Working Documentation (wrk_docs)

**Created:** 2025-11-06
**Purpose:** Comprehensive architecture analysis of the Stop the Bus codebase

---

## Document Index

### 🎯 Start Here

**[2025.11.06 - COMPREHENSIVE-ARCHITECTURE.md](2025.11.06%20-%20COMPREHENSIVE-ARCHITECTURE.md)**
- **Executive summary** of the entire project
- **System architecture** diagrams (ASCII)
- **Integration points** between all components
- **Data flow** diagrams for key processes
- **Index** to all other detailed documents

**Read this first** for the big-picture view, then dive into specific components as needed.

---

## Detailed Component Documentation

### 1. Legacy Codebase Analysis

**[2025.11.06 - legacy-pascal-analysis.md](2025.11.06%20-%20legacy-pascal-analysis.md)** (5,509 bytes)

Analysis of the original 1993 Turbo Pascal implementation:
- Version history (v1.0 → v1.11)
- Class hierarchy (TCardApp, TMainWindow, Dialogs)
- Game logic algorithms (Shuffle, Scoring, AI)
- Side-by-side comparison with Rust port
- Code location reference table

**Read this to understand:** The original Pascal implementation and how it maps to the modern Rust version.

---

### 2. Modern Rust Core Library

**[2025.11.06 - stopbus-core-architecture.md](2025.11.06%20-%20stopbus-core-architecture.md)** (Comprehensive)

Deep dive into the stopbus-core game logic library:
- Complete data structure documentation (GameState, DriveReport, GameEvent)
- Public API reference with usage examples
- Game algorithms (shuffling, scoring, AI logic, round completion)
- Internal state machine flow
- Testing infrastructure
- Comparison with Pascal implementation

**Read this to understand:** How the pure Rust game logic works independently of the UI.

---

### 3. Modern Rust UI Layer

**[2025.11.06 - stopbus-ui-architecture.md](2025.11.06%20-%20stopbus-ui-architecture.md)** (Comprehensive)

Deep dive into the stopbus-ui Win32 executable:
- Win32 message-driven architecture (WndProc handlers)
- WindowState structure and management
- GDI rendering pipeline
- Dialog procedures (Options, About)
- Cheat window system
- Registry persistence
- Integration with stopbus-core

**Read this to understand:** How the Win32 GUI layer works and interacts with the game logic.

---

### 4. Resource & Build Pipeline

**[2025.11.06 - resource-build-pipeline.md](2025.11.06%20-%20resource-build-pipeline.md)** (2,921 bytes)

Analysis of how resources flow through the build system:
- Dual resource architecture (legacy STOPBUS.RES + embedded)
- OS/2 → Windows V3 bitmap conversion process
- Build pipeline (build.rs) with timing metrics
- Resource ID mapping (1-54 for cards)
- Outstanding issues (WinHelp deprecation)

**Read this to understand:** How card bitmaps and resources are loaded and embedded.

---

### 5. Project Structure & Organization

**[2025.11.06 - project-structure-overview.md](2025.11.06%20-%20project-structure-overview.md)** (40,097 bytes)

Comprehensive overview of repository organization:
- Complete repository structure map
- Documentation ecosystem (README, AGENTS, CLAUDE.md)
- Development workflow and Git practices
- Migration timeline (Phases A-G) with progress tracking
- Historical artifacts and preservation practices
- Integration architecture

**Read this to understand:** How the entire repository is organized and how to navigate it.

---

## Document Statistics

| Document | Size | Focus |
|----------|------|-------|
| COMPREHENSIVE-ARCHITECTURE | ~50KB | Big picture + synthesis |
| stopbus-ui-architecture | ~40KB | Win32 GUI layer |
| project-structure-overview | 40KB | Repository organization |
| stopbus-core-architecture | ~35KB | Game logic library |
| legacy-pascal-analysis | 5.5KB | Original Pascal code |
| resource-build-pipeline | 2.9KB | Build system |
| **TOTAL** | **~173KB** | **Complete documentation** |

---

## How to Use This Documentation

### For New Contributors

1. Read **COMPREHENSIVE-ARCHITECTURE.md** first (executive summary)
2. Review **project-structure-overview.md** (understand repository layout)
3. Read **AGENTS.md** in root (contributor guidelines)
4. Dive into specific component docs as needed

### For Understanding Game Logic

1. Start with **stopbus-core-architecture.md** (Rust implementation)
2. Reference **legacy-pascal-analysis.md** (original algorithms)
3. Compare implementations to understand design decisions

### For Understanding UI

1. Start with **stopbus-ui-architecture.md** (Win32 layer)
2. Reference **resource-build-pipeline.md** (how resources work)
3. Check **legacy-pascal-analysis.md** (original UI patterns)

### For Understanding Build Process

1. Start with **resource-build-pipeline.md** (resource flow)
2. Reference **stopbus-ui-architecture.md** (build.rs details)
3. Check **project-structure-overview.md** (CI/CD pipeline)

---

## Visual Diagrams Included

All documents include ASCII diagrams for clarity:

**COMPREHENSIVE-ARCHITECTURE.md:**
- High-Level Component Diagram
- Rust Workspace Architecture
- Game Initialization Flow
- Human Turn Flow
- AI Turn Automation Flow
- Resource Loading Flow

**stopbus-core-architecture.md:**
- Game Flow State Machine
- Round End Conditions
- Game End Conditions

**stopbus-ui-architecture.md:**
- Win32 Component Hierarchy
- Message Handler Flow

---

## Related Documentation

**In Repository Root:**
- `README.md` - Quick start guide
- `CLAUDE.md` - Claude Code integration guide
- `AGENTS.md` - Repository guidelines
- `2025.09.20 - Migration & updating plan.md` - Phased roadmap

**In docs/ Directory:**
- `stopbus-architecture-notes.md` - Pascal code analysis
- `resource-id-map.md` - Resource ID reference
- `help-assets-inventory.md` - WinHelp migration planning
- `resource-embedding-plan.md` - Build strategy
- `card-bitmap-normalization-log.md` - Asset conversion log

---

## Document Freshness

**Created:** 2025-11-06 (all documents)
**Status:** Current as of Phase D (60% complete)
**Next Update:** Upon completion of Phase D or significant architecture changes

---

## Questions or Feedback

For questions about this documentation:
1. Check the comprehensive architecture doc first
2. Review the specific component doc
3. Consult the related docs in `docs/`
4. Open an issue if something is unclear or outdated

---

**Generated by:** Claude Code (Sonnet 4.5)
**Analysis Date:** 2025-11-06
