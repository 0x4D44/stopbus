# Card Bitmap Extraction Log

Date: 2025-09-22
Author: Codex agent (Python toolchain)

## Background

- The 16-bit `STOPBUS.RES` bundle stores the card art as BITMAPCOREHEADER
  (OS/2) payloads with 3-byte palettes and 16-bit row alignment.
- When Microsoft's resource toolchain embeds those payloads unchanged, it pads
  1-bpp images to DWORD boundaries using extra palette data, shifting the pixel
  stream. The result: red number cards and the card back render wrapped when the
  modern EXE loads them.

## Regeneration steps

- `tools/extract_cards.py` parses `STOPBUS.RES`, extracts every RT_BITMAP
  (IDs 1?54), and writes two outputs per card:
  - `assets/cards-os2/` ? archival BMPs that preserve the original headers and
    pixel data for provenance.
  - `crates/stopbus-ui/resources/cards/` ? Windows V3 BMPs with 40-byte DIB
    headers. The script now expands every 1-bpp asset to 4-bpp, generates a
    16-entry palette, and emits DWORD-aligned scanlines so `rc.exe` stops
    rewriting the data.
- Rerun the extractor whenever `STOPBUS.RES` changes:

  ```powershell
  python tools/extract_cards.py
  ```

## Verification

- `build.rs` enforces a >= 40-byte DIB header, so any regression back to the
  legacy format fails the build immediately.
- Inspect the regenerated EXE (or the exported BMPs) in Resource Hacker to
  confirm red cards/backdrops align correctly.

## Follow-up

- Keep `STOPBUS.RES` under version control; treat `assets/cards-os2/` as the raw
  source snapshots and avoid editing them by hand.
- Extend `extract_cards.py` if additional bitmap resources (e.g., help artwork)
  need conversion so the Rust build always depends on deterministic modern
  assets.
