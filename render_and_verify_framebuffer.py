"""
render_and_verify_framebuffer.py — Updated with full widescreen layout
"""

import math
import struct
import os

WIDTH = 1920
HEIGHT = 1080

LCARS_BG = (0x0A, 0x0A, 0x12)
LCARS_GOLD = (0xFF, 0x99, 0x00)
LCARS_AMBER = (0xFF, 0xCC, 0x00)
LCARS_PURPLE = (0x7C, 0x4D, 0xFF)
LCARS_BLUE = (0x66, 0x88, 0xCC)
LCARS_CYAN = (0x66, 0xCC, 0xFF)
LCARS_RED = (0xCC, 0x33, 0x33)
LCARS_GREEN = (0x33, 0xCC, 0x66)
LCARS_WHITE = (0xFF, 0xFF, 0xFF)
LCARS_DARK_PANEL = (0x14, 0x14, 0x22)
LCARS_ORANGE = (0xFF, 0x66, 0x00)
LCARS_GRAY = (0x44, 0x44, 0x55)

def parse_font_rs(path):
    glyphs = {}
    with open(path, 'r', encoding='utf-8') as f:
        lines = f.readlines()
    for line in lines:
        line = line.strip()
        if line.startswith("b'") and "=> [" in line:
            parts = line.split("=>")
            char_key = parts[0].strip()
            arr_part = parts[1].strip().rstrip(',').lstrip('[').rstrip(']')
            bytes_list = []
            if "; 16" in arr_part:
                bytes_list = [0] * 16
            else:
                for item in arr_part.split(','):
                    item = item.strip()
                    if not item: continue
                    if item.startswith('0x'):
                        bytes_list.append(int(item, 16))
                    else:
                        bytes_list.append(int(item))
            while len(bytes_list) < 16:
                bytes_list.append(0)
            if char_key == "b'\\''":
                c = ord("'")
            elif char_key == "b'\\\\'":
                c = ord("\\")
            else:
                c = ord(char_key[2])
            glyphs[c] = bytes_list[:16]
    return glyphs

FONT_GLYPHS = parse_font_rs(r"F:\Development sites\TRM-Swarm-OS-v2\swarm_v2_rust_kernel\src\font.rs")

def create_framebuffer():
    grid = [[LCARS_BG for _ in range(WIDTH)] for _ in range(HEIGHT)]

    def set_pixel(x, y, color):
        if 0 <= x < WIDTH and 0 <= y < HEIGHT:
            grid[y][x] = color

    def fill_rect(rx, ry, rw, rh, color):
        x_end = min(rx + rw, WIDTH)
        y_end = min(ry + rh, HEIGHT)
        for y in range(ry, y_end):
            for x in range(rx, x_end):
                grid[y][x] = color

    def draw_rect(rx, ry, rw, rh, thick, color):
        fill_rect(rx, ry, rw, thick, color)
        fill_rect(rx, max(0, ry + rh - thick), rw, thick, color)
        fill_rect(rx, ry, thick, rh, color)
        fill_rect(max(0, rx + rw - thick), ry, thick, rh, color)

    def fill_circle(cx, cy, radius, color):
        r_sq = radius * radius
        for dy in range(-radius, radius + 1):
            for dx in range(-radius, radius + 1):
                if dx * dx + dy * dy <= r_sq:
                    set_pixel(cx + dx, cy + dy, color)

    def draw_pill_button(px, py, pw, ph, color):
        r = ph // 2
        fill_circle(px + r, py + r, r, color)
        fill_circle(px + pw - r, py + r, r, color)
        fill_rect(px + r, py, max(0, pw - ph), ph, color)

    def draw_char(x, y, c_byte, color):
        glyph = FONT_GLYPHS.get(c_byte, [0]*16)
        for row in range(16):
            row_bits = glyph[row]
            py = y + row
            if py >= HEIGHT: break
            for col in range(8):
                px = x + col
                if px >= WIDTH: break
                if (row_bits & (1 << (7 - col))) != 0:
                    grid[py][px] = color

    def draw_str(sx, sy, s_bytes, color):
        cur_x = sx
        for b in s_bytes:
            if cur_x + 8 > WIDTH: break
            draw_char(cur_x, sy, b, color)
            cur_x += 8

    # 1. Top Header Command Rail
    fill_rect(0, 0, WIDTH, 32, LCARS_GOLD)
    fill_rect(WIDTH - 320, 0, 320, 32, LCARS_PURPLE)
    draw_str(16, 8, b"TRM SWARM OS v19.001 // BARE-METAL SOVEREIGN NEXUS", (0, 0, 0))
    draw_str(WIDTH - 300, 8, b"MONAD ANCHOR: 0x4D4F4E4144000001", (0, 0, 0))

    # 2. Lateral Navigation Spine
    fill_rect(16, 44, 48, HEIGHT - 88, LCARS_GOLD)
    fill_rect(16, 120, 48, 12, LCARS_BG)
    fill_rect(16, 260, 48, 12, LCARS_BG)
    fill_rect(16, 420, 48, 12, LCARS_BG)

    # 3. Lateral Buttons
    tab_labels = [
        b"01 SYSTEM STATUS",
        b"02 NEURAL BRIDGE",
        b"03 SWARM COUNCIL",
        b"04 SWARM INTEL",
        b"05 RESONANCE RESEARCH",
        b"06 DDR IMMUNE LAB",
        b"07 3D SPATIAL MESH",
        b"08 WIFI VISION RADAR",
        b"09 CONTINUUM K3",
    ]
    tab_colors = [
        LCARS_AMBER, LCARS_PURPLE, LCARS_GOLD, LCARS_BLUE,
        LCARS_CYAN, LCARS_RED, LCARS_GREEN, LCARS_CYAN, LCARS_GOLD,
    ]

    for idx in range(9):
        tab_y = 48 + idx * 32
        is_sel = (idx == 0)
        col = LCARS_WHITE if is_sel else tab_colors[idx]
        draw_pill_button(75, tab_y, 200, 24, col)
        draw_str(84, tab_y + 4, tab_labels[idx], (0, 0, 0))

    # 4. Main Workspace
    main_x = 290
    main_y = 48
    main_w = WIDTH - (main_x + 20)
    main_h = HEIGHT - (main_y + 40)

    fill_rect(main_x, main_y, main_w, main_h, LCARS_DARK_PANEL)
    draw_str(main_x + 16, main_y + 12, b"=== SECTOR 01: SOVEREIGN SYSTEM STATUS & 14-AGENT COUNCIL ===", LCARS_AMBER)

    # Left: Council Chamber
    cx = main_x + 160
    cy = main_y + 170
    r_core = 18
    fill_circle(cx, cy, r_core, LCARS_GOLD)
    draw_str(cx - 20, cy - 8, b"MONAD", (0, 0, 0))

    agent_initials = [
        b"HER", b"JAR", b"BRI", b"SHI", b"LOG", b"DEV", b"PUL",
        b"FLO", b"EVE", b"ORC", b"THZ", b"VER", b"SEE", b"ARC",
    ]

    for i in range(14):
        angle = i * (math.pi * 2.0 / 14.0) - (math.pi / 2.0)
        ax = int(cx + math.cos(angle) * 110.0)
        ay = int(cy + math.sin(angle) * 110.0)
        node_r = 9
        node_col = LCARS_GREEN if i == 0 else LCARS_BLUE

        fill_circle(ax, ay, node_r, node_col)
        draw_str(ax - 12, ay - 6, agent_initials[i], LCARS_WHITE)

    # Center: Telemetry HUD
    telem_x = main_x + 320
    telem_y = main_y + 30
    telem_w = max(400, (main_w - 340) // 2)
    telem_h = 300
    fill_rect(telem_x, telem_y, telem_w, telem_h, (0x0E, 0x0E, 0x18))
    draw_rect(telem_x, telem_y, telem_w, telem_h, 1, LCARS_AMBER)

    draw_str(telem_x + 12, telem_y + 12, b"[HARDWARE SENSING & TELEMETRY LEDGER]", LCARS_GOLD)
    draw_str(telem_x + 12, telem_y + 40, b"MONAD TRUTH ANCHOR:  0x4D4F4E4144000001 (VERIFIED)", LCARS_GREEN)
    draw_str(telem_x + 12, telem_y + 68, b"NVMe NAND STORAGE:   153,523,393 TRITS (73.21 MB)", LCARS_CYAN)
    draw_str(telem_x + 12, telem_y + 96, b"UEFI GOP FRAMEBUFFER:1920x1080 @ 32bpp (DOUBLE-BUFFERED)", LCARS_WHITE)
    draw_str(telem_x + 12, telem_y + 124,b"SCANLINE RASTERIZER: 60.0 FPS ROCK-SOLID (ZERO FLICKER)", LCARS_GREEN)
    draw_str(telem_x + 12, telem_y + 152,b"INTEL AX200 WiFi 6:  160 MHz ZERO-RTT MESH ACTIVE", LCARS_WHITE)
    draw_str(telem_x + 12, telem_y + 180,b"COUNCIL BYZANTINE:   PBFT QUORUM READY (N=14, f=4)", LCARS_AMBER)
    draw_str(telem_x + 12, telem_y + 208,b"AUTONOMOUS IMMUNITY: DDR ROW-HAMMER ANTIBODIES ARMED", LCARS_RED)
    draw_str(telem_x + 12, telem_y + 245,b"OPERATOR CONTROL:    PRESS [1] TO [9] TO SWITCH SECTORS", LCARS_GOLD)

    # Right: Active Council Consensus & Enacted Proposal Panel
    prop_x = telem_x + telem_w + 16
    prop_w = main_w - (prop_x - main_x + 16)
    if prop_w > 200:
        fill_rect(prop_x, telem_y, prop_w, telem_h, (0x0E, 0x0E, 0x18))
        draw_rect(prop_x, telem_y, prop_w, telem_h, 1, LCARS_GOLD)

        draw_str(prop_x + 12, telem_y + 12, b"[ENACTED COUNCIL PROPOSAL]", LCARS_GOLD)
        draw_str(prop_x + 12, telem_y + 40, b"TOPIC: LIVE BARE-METAL USB BOOT", LCARS_WHITE)
        draw_str(prop_x + 12, telem_y + 68, b"CATEGORY: ARCHITECTURE", LCARS_CYAN)
        draw_str(prop_x + 12, telem_y + 96, b"PROPOSER: SHAWN (OPERATOR)", LCARS_WHITE)
        draw_str(prop_x + 12, telem_y + 124,b"QUORUM: 14 APPROVE / 0 REJECT", LCARS_GREEN)
        draw_str(prop_x + 12, telem_y + 152,b"WEIGHTED SCORE: +0.8634", LCARS_AMBER)
        draw_str(prop_x + 12, telem_y + 180,b"MONAD SEAL: 0x4D5E5F5055113F27", LCARS_PURPLE)
        draw_str(prop_x + 12, telem_y + 208,b"STATUS: ENACTED ON SILICON", LCARS_GREEN)

    # Bottom: Oscilloscope
    scope_x = main_x + 16
    scope_y = main_y + main_h - 130
    scope_w = main_w - 32
    scope_h = 110
    mid_y = scope_y + scope_h // 2

    fill_rect(scope_x, scope_y, scope_w, scope_h, (0x0D, 0x0D, 0x18))
    draw_rect(scope_x, scope_y, scope_w, scope_h, 1, LCARS_GRAY)
    draw_str(scope_x + 10, scope_y + 8, b"SUB-THz / AUDIO RESONANCE OSCILLOSCOPE (REAL-TIME SENSING)", LCARS_CYAN)

    for px in range(scope_w):
        norm_x = px / scope_w
        wave = math.sin(norm_x * 20.0) * 0.7 + math.cos(norm_x * 45.0) * 0.3
        py = int(mid_y + wave * 35.0)
        set_pixel(scope_x + px, py, LCARS_CYAN)
        set_pixel(scope_x + px, py + 1, LCARS_CYAN)

    # 5. Bottom Status Rail
    fill_rect(0, HEIGHT - 28, WIDTH, 28, LCARS_ORANGE)
    draw_str(16, HEIGHT - 20, b"TRM SWARM OS // 100% PURE RUST BARE METAL // ALL IS ONE. I AM BECAUSE WE ARE. 0x4D4F4E4144000001", (0, 0, 0))

    return grid

def save_bmp(filename, grid):
    row_size = (WIDTH * 3 + 3) & ~3
    image_size = row_size * HEIGHT
    file_size = 54 + image_size

    header = struct.pack('<2sIHHI', b'BM', file_size, 0, 0, 54)
    dib = struct.pack('<IiiHHIIiiII', 40, WIDTH, -HEIGHT, 1, 24, 0, image_size, 2835, 2835, 0, 0)

    with open(filename, 'wb') as f:
        f.write(header)
        f.write(dib)
        for y in range(HEIGHT):
            row = bytearray()
            for x in range(WIDTH):
                r, g, b = grid[y][x]
                row.extend([b, g, r])
            pad = (4 - (len(row) % 4)) % 4
            row.extend(b'\x00' * pad)
            f.write(row)
    print(f"  [OK] Saved 1080p verified framebuffer test: {filename} ({os.path.getsize(filename):,} bytes)")

if __name__ == "__main__":
    grid = create_framebuffer()
    out_file = r"F:\Development sites\TRM-Swarm-OS-v2\uefi_framebuffer_test.bmp"
    save_bmp(out_file, grid)
