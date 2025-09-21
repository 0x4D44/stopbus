# Stop the Bus Architecture Notes (legacy Turbo Pascal build)

_Last reviewed: 20 September 2025_

## High-Level Structure
- `STOPBUS.PAS` defines the entire application: a `TCardApp` object (descends from `TApplication`) that hosts the main window, dialog implementations, and supporting child windows used for debugging/cheat features.
- UI is built with Turbo Vision for Windows (`WObjects`) atop classic Win16 APIs (`WinTypes`, `WinProcs`), with CTL3D providing themed controls. Message handling follows Borland’s `TWindow` virtual method model (`CM_*`, `WM_*`, `ID_*`).
- Game data lives in global arrays (`PackCard`, `Player`, `RoundScore`) rather than encapsulated objects; helper procedures on `TMainWindow` manipulate these globals directly.

## Core Types & Responsibilities
### Global Records & Arrays
- `TCard` record stores per-card state (`Card` ordinal index) plus default coordinates and a left-button flag, reused for each player's three cards (`Player[1..4,1..3]`).
- `PackCard[1..52]` mirrors deck order. `Cards[1..52]` and `CardBack` hold GDI handles for bitmap resources. `RoundScore[1..4]` caches the current round scores.
- `StackPntr` tracks top-of-stack index; `PlayerToPlay` indicates active player.

### `TCardApp`
- Overrides only `InitMainWindow`, instantiating `TMainWindow` and loading resources via `LoadAccel`/`LoadIcon`.

### Dialogs
- `TOptions` (`ID_OPTN*`): wraps four checkboxes controlling cheat overlays and “save on exit”. Handles `SetupWindow`, `OK`, and `Help` methods; persists settings through INI calls (see lines ~350–420).
- `TAboutBox`: populates static text controls (`ID_ABOTTITLE`, `ID_VERSIONNO`, etc.) with licensee information constants (`UserName`, `CompanyName`, `Address`). Uses `GetProfileString` to personalize when registration data exists.
- Cheat windows (`TCheatCards`, `TCheatScores`, `TCheatStack`) each derive from `TWindow`, override `GetWindowClass`, `SetupWindow`, `Paint`, and `WMMove`. They display hidden information depending on option toggles.

### `TMainWindow`
- Fields cover UI handles (`StickBut`, `DealBut`, etc.), cheat-window pointers, position tracking, and gameplay variables (`Lives[1..4]`, `StopBus`, `Playing`, `NotUsedNextCard`).
- `SetupWindow` loads card bitmaps from resources (`LoadBitmap` for `CARD01`-style IDs), creates child buttons, and posts `WM_START` to begin a session.
- Command handlers:
  - `CMGameDeal`: deals a new card to the active player, updates display, and checks for round completion.
  - `CMGameOptn`: launches `TOptions` dialog; toggles cheat windows via `HandleChCa/St/Sc`.
  - `CMGameExit`: saves options (if enabled) and closes.
  - `HelpCtnt`, `HelpUsin`, `HelpAbot`: bridge to WinHelp (`WinHelp` API) and spawn `TAboutBox`.
- Button handlers (`PR_*`) mirror menu commands; `PR_HappBut` plays an Easter egg sequence (animated bitmaps).
- Gameplay methods:
  - `Shuffle` randomizes `PackCard` and populates `Player` hands.
  - `Deal` moves the next card from `PackCard` into the player’s hand UI, handles drag/drop logic, and updates bitmaps.
  - `GameControl` enforces round transitions, life deductions, and “Stop the Bus” detection.
  - `Scores`, `DisplScores`, `MinScore`, `GetScore`, `StopTheBus` implement scoring rules (31 triggers bus stop; suits & values derived via `ValScore`, `Suit`).
  - `AutoPlay` / `AutoPlayOld` automate turns when cheats are active.
  - Paint helpers `DrawBMP`, `Stick`, `SetPosInd`, `DisplStrtPlayer` render cards, markers, and status text directly with GDI operations (`BitBlt`, `TextOut`).
- Messaging sequence: `WM_START` initializes a game by calling `StartGame` -> `StartRound`; `WM_JUSTCASE` provides a recovery path when threads desync, resetting state.

## Resource & UI Layout (`ABOUT.RC`)
- Defines an `ABOUTBOX` dialog (157×119 dialog units) with controls corresponding to IDs 550–555.
- NOTE: Caption currently reads “About Nonogram Solver”; this appears to be a copy/paste artifact and should be corrected during modernization.
- Icons `ICON_1` and `ICON_2` referenced; ensure they exist in the compiled resource file when migrating.

## External Files & Tooling Hooks
- `RESOURCE.BAT` invokes `rc stopbus.res` via a TPW utilities path; modernization should replace with a portable build script.
- `HELP/STOPBUS.HPJ` defines WinHelp project settings linking `STOPBUS.RTF` and `BMP.ZIP` assets. Diagnostics live in `STOPBUS.ERR` and `STOPBUS.GID`.
- `CTL3D.PAS` supplies wrappers around CTL3D API calls so the Pascal UI can register/unregister 3D controls during `SetupWindow`/`Done` in `TMainWindow`.

## Observations for Rust Port
- Message-driven architecture maps cleanly to Rust’s windows-rs `WndProc` pattern; each Pascal method corresponds to a case in the message loop.
- Globals will need encapsulation: consider a `GameState` struct with deck, players, and resource handles, owned by the main window state object.
- Cheat windows and helper dialogs can be optional features; align them with legacy INI flags for parity.
- Resource IDs (e.g., card bitmaps) should be enumerated in a Rust module to maintain mapping between numeric IDs and assets.
- Help experience requires replacement (WinHelp is deprecated); capture topics from `STOPBUS.RTF` for conversion to HTML/Markdown.

## Open Questions / Follow-Up
- Confirm location/naming of bitmap resources within `STOPBUS.RES` (e.g., card faces). Need extraction to ensure numbering scheme.
- Determine how registration data was intended to be saved (`WriteProfileString` usage) and whether to maintain compatibility.
- Validate whether CTL3D.DLL needs to ship alongside the modern build or if a modern theming approach supersedes it.
- Investigate `STOPBUS.BAK`/`STOPBUS.~RE` differences vs. current `STOPBUS.PAS` to ensure no logic is lost.


