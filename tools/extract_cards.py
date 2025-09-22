from __future__ import annotations
import struct
from pathlib import Path

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
            palette_os2[i:i+3] + b'\x00' for i in range(0, len(palette_os2), 3)
        )
        dib_header = struct.pack(
            '<IiiHHIIiiII',
            40,
            width,
            height,
            planes,
            bitcount,
            0,
            len(pixel_data_adj),
            0,
            0,
            len(palette_os2) // 3 if bitcount <= 8 else 0,
            0,
        )
        dib = dib_header + palette_rgba + pixel_data_adj
        os2_file_header = struct.pack('<2sIHHI', b'BM', len(payload) + 14, 0, 0, 14 + header_size)
        os2_file = os2_file_header + payload
        modern_file_header = struct.pack('<2sIHHI', b'BM', 14 + len(dib), 0, 0, 14 + len(dib) - len(pixel_data_adj))
        modern_file = modern_file_header + dib
        return (width, height, bitcount, os2_file, modern_file)
    else:
        width, height, planes, bitcount, compression, image_size = struct.unpack_from('<iiHHII', payload, 4)
        os2_file_header = struct.pack('<2sIHHI', b'BM', len(payload) + 14, 0, 0, 14 + header_size)
        os2_file = os2_file_header + payload
        modern_file_header = struct.pack('<2sIHHI', b'BM', 14 + len(payload), 0, 0, 14 + header_size)
        modern_file = modern_file_header + payload
        return (width, height, bitcount, os2_file, modern_file)

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
