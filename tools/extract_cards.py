from __future__ import annotations
import struct
from pathlib import Path
from typing import List, Tuple

RES_TYPE_BITMAP = 2
RES_PATH = Path('STOPBUS.RES')
OUT_MODERN = Path('crates/stopbus-ui/resources/cards')
OUT_OS2 = Path('assets/cards-os2')

OUT_MODERN.mkdir(parents=True, exist_ok=True)
OUT_OS2.mkdir(parents=True, exist_ok=True)

def read_res_entries(data: bytes):
    offset = 0
    while offset < len(data):
        type_tag = data[offset]
        if type_tag == 0x00:
            break
        if type_tag == 0xFF:
            res_type = struct.unpack_from('<H', data, offset + 1)[0]
            offset += 3
        else:
            end = data.index(0, offset)
            res_type = data[offset:end].decode('ascii')
            offset = end + 1
        name_tag = data[offset]
        if name_tag == 0xFF:
            res_id = struct.unpack_from('<H', data, offset + 1)[0]
            offset += 3
        else:
            end = data.index(0, offset)
            res_id = data[offset:end].decode('ascii')
            offset = end + 1
        flags = struct.unpack_from('<H', data, offset)[0]
        offset += 2
        size = struct.unpack_from('<I', data, offset)[0]
        offset += 4
        payload = data[offset:offset + size]
        offset += size
        if offset % 2:
            offset += 1
        yield res_type, res_id, flags, payload

def ensure_two_colors(colors: List[bytes]) -> List[bytes]:
    if not colors:
        return [b'\x00\x00\x00', b'\xFF\xFF\xFF']
    if len(colors) == 1:
        colors.append(colors[0])
    return colors

def palette_os2_to_list(palette_os2: bytes) -> List[bytes]:
    return [palette_os2[i:i + 3] for i in range(0, len(palette_os2), 3)]

def palette_rgba_to_list(palette_rgba: bytes) -> List[bytes]:
    return [palette_rgba[i:i + 3] for i in range(0, len(palette_rgba), 4)]

def make_4bpp_palette(colors: List[bytes], total_entries: int = 16) -> bytes:
    colors = ensure_two_colors(colors[:])
    fill_color = colors[1]
    entries = [colors[0] + b'\x00', colors[1] + b'\x00']
    while len(entries) < total_entries:
        entries.append(fill_color + b'\x00')
    return b''.join(entries)

def expand_1bpp_to_4bpp(
    width: int,
    height: int,
    pixel_data: bytes,
    stride: int,
    significant: int,
) -> Tuple[bytes, int]:
    new_stride = ((width * 4 + 31) // 32) * 4
    rows = []
    for row in range(height):
        row_start = row * stride
        row_bits = pixel_data[row_start:row_start + significant]
        pixel_values = []
        for byte in row_bits:
            for bit in range(7, -1, -1):
                if len(pixel_values) == width:
                    break
                pixel_values.append((byte >> bit) & 0x01)
        packed = bytearray()
        for idx in range(0, width, 2):
            hi = pixel_values[idx]
            lo = pixel_values[idx + 1] if idx + 1 < width else hi
            packed.append((hi << 4) | lo)
        padding = new_stride - len(packed)
        if padding < 0:
            raise ValueError('negative padding calculated during 1-bpp expansion')
        rows.append(bytes(packed) + (b'\x00' * padding))
    return b''.join(rows), new_stride

def build_bitmapfile_from_core(payload: bytes):
    header_size = struct.unpack_from('<I', payload, 0)[0]
    if header_size == 12:
        width, height, planes, bitcount = struct.unpack_from('<HHHH', payload, 4)
        palette_entries = 1 << bitcount if bitcount <= 8 else 0
        palette_offset = 12
        palette_os2 = payload[palette_offset:palette_offset + palette_entries * 3]
        pixel_offset = palette_offset + palette_entries * 3
        pixel_data = payload[pixel_offset:]
        significant = (width * bitcount + 7) // 8
        stride = ((width * bitcount + 15) // 16) * 2
        if stride * height != len(pixel_data) and len(pixel_data) % height == 0:
            stride = len(pixel_data) // height
        if stride * height != len(pixel_data):
            raise ValueError(f'core bitmap stride mismatch (len={len(pixel_data)}, height={height})')

        os2_file_header = struct.pack('<2sIHHI', b'BM', len(payload) + 14, 0, 0, 14 + header_size)
        os2_file = os2_file_header + payload

        if bitcount == 1:
            pixel_data_adj, _ = expand_1bpp_to_4bpp(width, height, pixel_data, stride, significant)
            palette_rgba = make_4bpp_palette(palette_os2_to_list(palette_os2))
            bi_clr_used = 16
            bitcount_for_header = 4
        else:
            new_stride = ((width * bitcount + 31) // 32) * 4
            rows = []
            for row in range(height):
                start = row * stride
                row_pixels = pixel_data[start:start + significant]
                padding = new_stride - significant
                if padding < 0:
                    raise ValueError('negative padding calculated')
                rows.append(row_pixels + (b'\x00' * padding))
            pixel_data_adj = b''.join(rows)
            palette_rgba = b''.join(
                palette_os2[i:i + 3] + b'\x00' for i in range(0, len(palette_os2), 3)
            )
            bi_clr_used = len(palette_os2) // 3 if bitcount <= 8 else 0
            bitcount_for_header = bitcount

        dib_header = struct.pack(
            '<IiiHHIIiiII',
            40,
            width,
            height,
            planes,
            bitcount_for_header,
            0,
            len(pixel_data_adj),
            0,
            0,
            bi_clr_used,
            0,
        )
        dib = dib_header + palette_rgba + pixel_data_adj
        modern_file_header = struct.pack(
            '<2sIHHI',
            b'BM',
            14 + len(dib),
            0,
            0,
            14 + len(dib) - len(pixel_data_adj),
        )
        modern_file = modern_file_header + dib
        return (width, height, bitcount_for_header, os2_file, modern_file)

    width, height, planes, bitcount, compression, image_size = struct.unpack_from('<iiHHII', payload, 4)
    palette_entries = struct.unpack_from('<I', payload, 32)[0]
    if palette_entries == 0 and bitcount <= 8:
        palette_entries = 1 << bitcount
    palette_offset = header_size
    palette_bytes = payload[palette_offset:palette_offset + palette_entries * 4]
    pixel_offset = palette_offset + palette_entries * 4
    pixel_data = payload[pixel_offset:]
    significant = (width * bitcount + 7) // 8
    stride = ((width * bitcount + 31) // 32) * 4

    os2_file_header = struct.pack('<2sIHHI', b'BM', len(payload) + 14, 0, 0, 14 + header_size)
    os2_file = os2_file_header + payload

    if bitcount == 1:
        pixel_data_adj, _ = expand_1bpp_to_4bpp(width, height, pixel_data, stride, significant)
        palette_rgba = make_4bpp_palette(palette_rgba_to_list(palette_bytes))
        bi_clr_used = 16
        bitcount_for_header = 4
    else:
        pixel_data_adj = pixel_data
        palette_rgba = palette_bytes
        bi_clr_used = palette_entries if bitcount <= 8 else 0
        bitcount_for_header = bitcount

    dib_header = struct.pack(
        '<IiiHHIIiiII',
        40,
        width,
        height,
        planes,
        bitcount_for_header,
        compression,
        len(pixel_data_adj) if compression == 0 else image_size,
        0,
        0,
        bi_clr_used,
        0,
    )
    dib = dib_header + palette_rgba + pixel_data_adj
    modern_file_header = struct.pack(
        '<2sIHHI',
        b'BM',
        14 + len(dib),
        0,
        0,
        14 + len(dib) - len(pixel_data_adj),
    )
    modern_file = modern_file_header + dib
    return (width, height, bitcount_for_header, os2_file, modern_file)

def main() -> None:
    data = RES_PATH.read_bytes()
    count = 0
    for res_type, res_id, _flags, payload in read_res_entries(data):
        if res_type != RES_TYPE_BITMAP or not isinstance(res_id, int):
            continue
        width, height, bitcount, os2_bmp, modern_bmp = build_bitmapfile_from_core(payload)
        (OUT_OS2 / f'card{res_id:02d}.bmp').write_bytes(os2_bmp)
        (OUT_MODERN / f'card{res_id:02d}.bmp').write_bytes(modern_bmp)
        print(f'Card {res_id:02d}: {width}x{height}, {bitcount}-bpp')
        count += 1
    print(f'Extracted {count} card bitmaps from {RES_PATH}')

if __name__ == '__main__':
    main()
