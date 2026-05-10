#!/usr/bin/env python3
"""
Generates images/claude.gif — a 16×16 pixel Clawd mascot (the 8-bit crab from Claude Code).
Then converts it to images/claude.divoom16 using the project binary.

Color spec: #D77357 (rgb 215,119,87) on black background, traced from the Clawd reference.

Run from the project root:
    python3 scripts/create_mascot.py
"""

from __future__ import annotations

import os
import struct
import subprocess
import sys
from typing import List, Tuple

ORANGE = (215, 119, 87)
BLACK  = (0, 0, 0)

# 16×16 pixel grid — O=orange, .=black
# Traced from ~/Downloads/HAkuSWBaMAE2ztz.jpg (official Clawd reference image)
GRID = [
    "................",  # row  0
    "...OOOOOOOOOO...",  # row  1  body top (10 wide, cols 3-12)
    "...OOOOOOOOOO...",  # row  2
    "...OO..OO..OO...",  # row  3  eyes: 2×2 black holes at cols 5-6 and 9-10
    "...OO..OO..OO...",  # row  4
    "..OOOOOOOOOOOO..",  # row  5  side protrusions (12 wide, cols 2-13)
    "..OOOOOOOOOOOO..",  # row  6
    "...OOOOOOOOOO...",  # row  7  lower body (back to 10 wide)
    "...OOOOOOOOOO...",  # row  8
    "....O.O..O.O....",  # row  9  4 leg stumps at cols 4,6,9,11
    "....O.O..O.O....",  # row 10
    "................",  # row 11
    "................",  # row 12
    "................",  # row 13
    "................",  # row 14
    "................",  # row 15
]

assert len(GRID) == 16, "GRID must have 16 rows"
for i, row in enumerate(GRID):
    assert len(row) == 16, f"Row {i} must be 16 chars wide, got {len(row)}"


def make_gif(pixels: List[Tuple[int,int,int]], width: int, height: int) -> bytes:
    """Build a minimal single-frame GIF89a with a 2-colour palette."""

    palette = [BLACK, ORANGE]
    palette_bytes = bytearray()
    for r, g, b in palette:
        palette_bytes += bytes([r, g, b])
    # GIF palettes are always a power-of-2 size; pad to 4 entries (12 bytes)
    palette_bytes += bytes(6)  # 2 unused entries × 3 bytes

    def le16(n: int) -> bytes:
        return struct.pack("<H", n)

    # Header
    header = b"GIF89a"

    # Logical Screen Descriptor
    # size field encodes colour table size: bits 2-0 = (2^(n+1) colours), n=1 → 4 entries
    lsd = (le16(width) + le16(height)
           + bytes([0b10000001, 0, 0]))  # global CT flag, 1bpp (4 colours), bg=0, AR=0

    # Image Descriptor
    img_desc = (b"\x2C"
                + le16(0) + le16(0)      # left, top
                + le16(width) + le16(height)
                + bytes([0]))            # no local CT, not interlaced

    # Build pixel index stream (1 bit per pixel packed into LZW codes)
    indices = []
    for px in pixels:
        indices.append(1 if px == ORANGE else 0)

    # Minimal LZW encoder (lzw_min_code_size = 2 for a 2-colour palette in GIF)
    lzw_min = 2
    lzw_data = _lzw_compress(indices, lzw_min)

    # Pack LZW data into sub-blocks (max 255 bytes each)
    sub_blocks = bytearray()
    sub_blocks += bytes([lzw_min])
    i = 0
    while i < len(lzw_data):
        chunk = lzw_data[i:i+255]
        sub_blocks += bytes([len(chunk)]) + chunk
        i += 255
    sub_blocks += b"\x00"  # block terminator

    trailer = b"\x3B"

    return (header + lsd + bytes(palette_bytes)
            + img_desc + bytes(sub_blocks) + trailer)


def _lzw_compress(indices: list[int], min_code_size: int) -> bytes:
    """Standard GIF LZW compression, returns packed bit stream as bytes."""
    clear_code = 1 << min_code_size
    eoi_code   = clear_code + 1
    code_size  = min_code_size + 1
    next_code  = eoi_code + 1

    table: dict[tuple, int] = {(i,): i for i in range(clear_code)}
    output_bits = []

    def emit(code: int, bits: int) -> None:
        for b in range(bits):
            output_bits.append((code >> b) & 1)

    emit(clear_code, code_size)

    buf = ()
    for idx in indices:
        candidate = buf + (idx,)
        if candidate in table:
            buf = candidate
        else:
            emit(table[buf], code_size)
            if next_code < 4096:
                table[candidate] = next_code
                next_code += 1
                if next_code > (1 << code_size) and code_size < 12:
                    code_size += 1
            else:
                emit(clear_code, code_size)
                code_size = min_code_size + 1
                next_code = eoi_code + 1
                table = {(i,): i for i in range(clear_code)}
            buf = (idx,)

    if buf:
        emit(table[buf], code_size)
    emit(eoi_code, code_size)

    # Pack bits (LSB first) into bytes
    result = bytearray()
    for i in range(0, len(output_bits), 8):
        byte = 0
        for b, bit in enumerate(output_bits[i:i+8]):
            byte |= bit << b
        result.append(byte)
    return bytes(result)


def main() -> None:
    project_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    gif_path     = os.path.join(project_root, "images", "claude.gif")
    div_path     = os.path.join(project_root, "images", "claude.divoom16")
    binary       = os.path.join(project_root, "target", "release",
                                "divoom-ditoo-pro-controller")

    # Build pixel list from GRID
    pixels: List[Tuple[int,int,int]] = []
    for row in GRID:
        for ch in row:
            pixels.append(ORANGE if ch == "O" else BLACK)

    gif_bytes = make_gif(pixels, 16, 16)
    os.makedirs(os.path.dirname(gif_path), exist_ok=True)
    with open(gif_path, "wb") as f:
        f.write(gif_bytes)
    print(f"Wrote {gif_path} ({len(gif_bytes)} bytes)")

    if not os.path.exists(binary):
        print(f"Binary not found at {binary} — run 'make build' first, then re-run this script.")
        sys.exit(0)

    result = subprocess.run(
        [binary, "convert", "to-divoom16", gif_path, div_path],
        capture_output=True, text=True
    )
    if result.returncode != 0:
        print(f"Conversion failed: {result.stderr}")
        sys.exit(1)
    print(f"Wrote {div_path}")


if __name__ == "__main__":
    main()
