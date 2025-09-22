# Card Bitmap Extraction Log

Date: 2025-09-22
Author: Codex agent (Python toolchain)

## Background

- The shipping STOPBUS assets store card art inside the 16-bit resource bundle
  `STOPBUS.RES` as BITMAPCOREHEADER payloads (OS/2 style with 3-byte palettes and
  16-bit row alignment).
- Feeding those payloads directly to `rc.exe` leaves the headers unchanged, so
  modern Windows releases treat the palette entries as 4-byte blocks and rows as
  DWORD-aligned. Without conversion, scanlines can start mid-row when rendered.

## Regeneration steps

- Added `tools/extract_cards.py`. It parses `STOPBUS.RES`, pulls each RT_BITMAP
  entry (IDs 1?54), and writes two outputs per card:
  - `assets/cards-os2/` ? archival copies that preserve the legacy OS/2 layout
    (BITMAPFILEHEADER + BITMAPCOREHEADER + original palette/pixels).
  - `crates/stopbus-ui/resources/cards/` ? Windows V3 BMPs with 40-byte DIB
    headers, DWORD-aligned rows, and 4-byte palette entries ready for `rc.exe`.
- The script is deterministic; rerun it any time `STOPBUS.RES` changes:

  ```powershell
  python tools/extract_cards.py
  ```

## Verification

- `build.rs` asserts each bitmap uses a >=40-byte DIB header, so stale OS/2
  payloads fail the build immediately.
- Inspect the regenerated EXE or the exported BMPs (e.g., via Resource Hacker) to
  confirm assets render with correct alignment.

## Follow-up

- Keep `STOPBUS.RES` under version control; treat `assets/cards-os2/` as the raw
  provenance snapshots and avoid editing them by hand.
- If additional bitmap resources are needed (e.g., help artwork), extend
  `extract_cards.py` to target the new resource IDs so the Windows build always
  derives assets from the legacy binary.
