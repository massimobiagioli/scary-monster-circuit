#!/usr/bin/env python3
"""Draw breadboard-style assembly pictures for the Scary Monster Circuit.

Each picture is a top view of the breadboard showing the parts and the coloured
jumper wires for one build step. The pictures are written as PNG files and are
embedded in the step-by-step Markdown document.
"""

from __future__ import annotations

import argparse
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

WIDTH, HEIGHT = 1500, 1060

BG = (255, 255, 255)
BOARD = (243, 243, 243)
BOARD_EDGE = (205, 205, 205)
HOLE = (170, 170, 170)
GAP = (232, 232, 232)
RED = (219, 68, 55)
BLUE = (52, 101, 196)
PICO = (46, 125, 50)
PICO_EDGE = (27, 94, 32)
GOLD = (240, 196, 15)
DARK = (45, 45, 45)
TEXT = (25, 25, 25)
NEW = (255, 193, 7)

WIRE_5V = (219, 68, 55)
WIRE_GND = (35, 35, 35)
WIRE_TRIG = (255, 152, 0)
WIRE_ECHO = (0, 150, 136)
WIRE_SIGNAL = (255, 152, 0)
WIRE_TX = (63, 81, 181)
WIRE_RX = (156, 39, 176)
WIRE_AUDIO = (120, 120, 120)

FONT_CANDIDATES = [
    "/System/Library/Fonts/Supplemental/Arial.ttf",
    "/Library/Fonts/Arial.ttf",
    "/System/Library/Fonts/Helvetica.ttc",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
]
BOLD_CANDIDATES = [
    "/System/Library/Fonts/Supplemental/Arial Bold.ttf",
    "/Library/Fonts/Arial Bold.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
]


def load_font(candidates, size):
    for path in candidates:
        if Path(path).exists():
            return ImageFont.truetype(path, size)
    return ImageFont.load_default()


FONT = load_font(FONT_CANDIDATES, 20)
FONT_SMALL = load_font(FONT_CANDIDATES, 15)
FONT_PIN = load_font(FONT_CANDIDATES, 14)
FONT_BOLD = load_font(BOLD_CANDIDATES, 22)
FONT_TITLE = load_font(BOLD_CANDIDATES, 34)


def rounded(draw, box, radius, fill, outline=None, width=1):
    draw.rounded_rectangle(box, radius=radius, fill=fill, outline=outline, width=width)


def text(draw, xy, value, font=FONT, fill=TEXT, anchor="mm"):
    draw.text(xy, value, font=font, fill=fill, anchor=anchor)


def text_box(draw, xy, value, font=FONT_SMALL, fill=TEXT, pad=4):
    box = draw.textbbox(xy, value, font=font, anchor="mm")
    draw.rounded_rectangle(
        (box[0] - pad, box[1] - pad, box[2] + pad, box[3] + pad),
        radius=6,
        fill=(255, 255, 255),
        outline=(210, 210, 210),
        width=1,
    )
    draw.text(xy, value, font=font, fill=fill, anchor="mm")


def dashed_rect(draw, box, color=NEW, width=3, dash=12):
    x0, y0, x1, y1 = box
    for x in range(x0, x1, dash * 2):
        draw.line((x, y0, min(x + dash, x1), y0), fill=color, width=width)
        draw.line((x, y1, min(x + dash, x1), y1), fill=color, width=width)
    for y in range(y0, y1, dash * 2):
        draw.line((x0, y, x0, min(y + dash, y1)), fill=color, width=width)
        draw.line((x1, y, x1, min(y + dash, y1)), fill=color, width=width)


def bezier(p0, p1, p2, p3, steps=48):
    points = []
    for i in range(steps + 1):
        t = i / steps
        mt = 1 - t
        x = mt**3 * p0[0] + 3 * mt**2 * t * p1[0] + 3 * mt * t**2 * p2[0] + t**3 * p3[0]
        y = mt**3 * p0[1] + 3 * mt**2 * t * p1[1] + 3 * mt * t**2 * p2[1] + t**3 * p3[1]
        points.append((x, y))
    return points


def wire(draw, start, end, color, label=None, width=6):
    dx = end[0] - start[0]
    dy = end[1] - start[1]
    bend = max(abs(dy) * 0.45, 40)
    c1 = (start[0] + dx * 0.15, start[1] + bend if dy >= 0 else start[1] - bend)
    c2 = (end[0] - dx * 0.15, end[1] - bend if dy >= 0 else end[1] + bend)
    points = bezier(start, c1, c2, end)
    draw.line(points, fill=color, width=width, joint="curve")
    draw.ellipse(
        (start[0] - width, start[1] - width, start[0] + width, start[1] + width),
        fill=color,
    )
    draw.ellipse(
        (end[0] - width, end[1] - width, end[0] + width, end[1] + width),
        fill=color,
    )
    if label:
        mid = points[len(points) // 2]
        text_box(draw, mid, label)


def draw_breadboard(draw):
    rounded(draw, (70, 120, 1330, 900), 26, BOARD, BOARD_EDGE, 4)
    for x0, x1, y, color in [
        (100, 1300, 172, RED),
        (100, 1300, 207, BLUE),
        (100, 1300, 800, BLUE),
        (100, 1300, 835, RED),
    ]:
        draw.rounded_rectangle((x0, y - 10, x1, y + 10), radius=8, fill=color)
        for x in range(x0 + 12, x1, 26):
            draw.ellipse((x - 3, y - 3, x + 3, y + 3), fill=(255, 255, 255))
    draw.rectangle((90, 470, 1310, 530), fill=GAP)
    for y in (270, 310, 350, 390, 430, 560, 600, 640, 680, 720):
        for x in range(110, 1310, 26):
            draw.ellipse((x - 3, y - 3, x + 3, y + 3), fill=HOLE)
    text(draw, (125, 172), "+ 5V", FONT_PIN, (255, 255, 255), "lm")
    text(draw, (125, 207), "- GND", FONT_PIN, (255, 255, 255), "lm")


def draw_pico(draw):
    rounded(draw, (480, 448, 1020, 552), 16, PICO, PICO_EDGE, 4)
    draw.rounded_rectangle((418, 476, 486, 524), radius=6, fill=(200, 200, 205))
    text(draw, (452, 500), "USB", FONT_PIN, DARK)
    text(draw, (760, 500), "Raspberry Pi Pico", FONT_BOLD, (255, 255, 255))
    top_pins = {
        "GP2": 560,
        "GP3": 600,
        "GP4": 640,
        "GP5": 680,
        "VSYS": 740,
        "GND": 790,
    }
    bottom_pins = {"GP16": 760, "GP0": 900, "GP1": 945}
    anchors = {}
    for name, x in top_pins.items():
        draw.rectangle((x - 5, 440, x + 5, 456), fill=GOLD)
        text(draw, (x, 468), name, FONT_PIN, (255, 255, 255), "mm")
        anchors[name] = (x, 448)
    for name, x in bottom_pins.items():
        draw.rectangle((x - 5, 544, x + 5, 560), fill=GOLD)
        text(draw, (x, 532), name, FONT_PIN, (255, 255, 255), "mm")
        anchors[name] = (x, 552)
    return anchors


def draw_sensor(draw, origin, mirrored=False):
    x0, y0 = origin
    w, h = 180, 150
    rounded(draw, (x0, y0, x0 + w, y0 + h), 12, (33, 150, 243), (21, 101, 192), 3)
    for cx in (x0 + 55, x0 + 125):
        draw.ellipse((cx - 26, y0 + 20, cx + 26, y0 + 72), fill=(190, 190, 190))
        draw.ellipse((cx - 18, y0 + 28, cx + 18, y0 + 64), fill=(90, 90, 90))
    text(draw, (x0 + w / 2, y0 + 112), "distance sensor", FONT_SMALL, (255, 255, 255))
    pins = ["VCC", "TRIG", "ECHO", "GND"]
    anchors = {}
    for i, name in enumerate(pins):
        px = x0 + 30 + i * 40
        py = y0 + h
        draw.rectangle((px - 5, py - 6, px + 5, py + 6), fill=GOLD)
        text(draw, (px, py + 20), name, FONT_PIN, TEXT)
        anchors[name] = (px, py)
    return anchors


def draw_servo(draw, origin):
    x0, y0 = origin
    w, h = 150, 140
    rounded(draw, (x0, y0, x0 + w, y0 + h), 12, (96, 125, 139), (55, 71, 79), 3)
    draw.ellipse((x0 + 45, y0 + 20, x0 + 105, y0 + 80), fill=(207, 216, 220))
    text(draw, (x0 + w / 2, y0 + 108), "servo", FONT_SMALL, (255, 255, 255))
    anchors = {}
    for i, (name, color) in enumerate(
        [("signal", WIRE_SIGNAL), ("+5V", WIRE_5V), ("GND", WIRE_GND)]
    ):
        px = x0 + 35 + i * 40
        py = y0
        draw.rectangle((px - 5, py - 6, px + 5, py + 6), fill=color)
        text(draw, (px, py - 20), name, FONT_PIN, TEXT)
        anchors[name] = (px, py)
    return anchors


def draw_mp3(draw, origin):
    x0, y0 = origin
    w, h = 180, 140
    rounded(draw, (x0, y0, x0 + w, y0 + h), 12, (55, 71, 79), (33, 43, 48), 3)
    draw.rounded_rectangle((x0 + 20, y0 + 20, x0 + 90, y0 + 60), radius=4, fill=(120, 144, 156))
    draw.rectangle((x0 + 110, y0 + 20, x0 + 160, y0 + 60), fill=(189, 189, 189))
    text(draw, (x0 + w / 2, y0 + 100), "MP3 player", FONT_SMALL, (255, 255, 255))
    pins = ["VCC", "GND", "RX", "TX"]
    anchors = {}
    for i, name in enumerate(pins):
        px = x0 + 30 + i * 40
        py = y0
        draw.rectangle((px - 5, py - 6, px + 5, py + 6), fill=GOLD)
        text(draw, (px, py - 20), name, FONT_PIN, TEXT)
        anchors[name] = (px, py)
    return anchors


def draw_speaker(draw, center):
    cx, cy = center
    draw.ellipse((cx - 60, cy - 60, cx + 60, cy + 60), fill=(120, 144, 156), outline=(69, 90, 100), width=3)
    draw.ellipse((cx - 26, cy - 26, cx + 26, cy + 26), fill=(69, 90, 100))
    draw.polygon(
        [(cx - 60, cy - 30), (cx - 110, cy - 60), (cx - 110, cy + 60), (cx - 60, cy + 30)],
        fill=(96, 125, 139),
        outline=(69, 90, 100),
    )
    text(draw, (cx, cy + 78), "speaker", FONT_SMALL, TEXT)


def draw_power(draw, origin):
    x0, y0 = origin
    w, h = 170, 110
    rounded(draw, (x0, y0, x0 + w, y0 + h), 12, (69, 90, 100), (38, 50, 56), 3)
    text(draw, (x0 + w / 2, y0 + 40), "5V power", FONT_BOLD, (255, 255, 255))
    text(draw, (x0 + w / 2, y0 + 72), "supply", FONT, (255, 255, 255))
    return {
        "+": (x0 + 40, y0),
        "-": (x0 + 130, y0),
    }


def render_step(step: int, destination: Path) -> None:
    image = Image.new("RGB", (WIDTH, HEIGHT), BG)
    draw = ImageDraw.Draw(image)
    draw_breadboard(draw)

    titles = {
        1: "Step 1 - Place the Pico on the breadboard",
        2: "Step 2 - Connect the 5V power rails",
        3: "Step 3 - Add the LEFT distance sensor",
        4: "Step 4 - Add the RIGHT distance sensor",
        5: "Step 5 - Add the head servo",
        6: "Step 6 - Add the MP3 player and speaker",
        7: "Step 7 - The finished monster",
    }
    text(draw, (WIDTH / 2, 60), titles.get(step, ""), FONT_TITLE, TEXT)

    pico = draw_pico(draw)

    if step >= 2:
        power = draw_power(draw, (110, 940))
        wire(draw, power["+"], (200, 835), WIRE_5V, "+5V")
        wire(draw, power["-"], (300, 800), WIRE_GND, "GND")
        wire(draw, pico["VSYS"], (740, 172), WIRE_5V, "VSYS")
        wire(draw, pico["GND"], (790, 207), WIRE_GND, "GND")

    if step >= 3:
        left = draw_sensor(draw, (150, 250))
        wire(draw, left["VCC"], (200, 172), WIRE_5V, "5V")
        wire(draw, left["GND"], (330, 207), WIRE_GND, "GND")
        wire(draw, left["TRIG"], pico["GP2"], WIRE_TRIG, "TRIG")
        wire(draw, left["ECHO"], pico["GP3"], WIRE_ECHO, "ECHO")

    if step >= 4:
        right = draw_sensor(draw, (1070, 250))
        wire(draw, right["VCC"], (1200, 172), WIRE_5V, "5V")
        wire(draw, right["GND"], (1080, 207), WIRE_GND, "GND")
        wire(draw, right["TRIG"], pico["GP4"], WIRE_TRIG, "TRIG")
        wire(draw, right["ECHO"], pico["GP5"], WIRE_ECHO, "ECHO")

    if step >= 5:
        servo = draw_servo(draw, (170, 660))
        wire(draw, servo["signal"], pico["GP16"], WIRE_SIGNAL, "signal")
        wire(draw, servo["+5V"], (240, 835), WIRE_5V, "5V")
        wire(draw, servo["GND"], (300, 800), WIRE_GND, "GND")

    if step >= 6:
        mp3 = draw_mp3(draw, (1070, 660))
        wire(draw, mp3["VCC"], (1200, 835), WIRE_5V, "5V")
        wire(draw, mp3["GND"], (1100, 800), WIRE_GND, "GND")
        wire(draw, mp3["RX"], pico["GP0"], WIRE_TX, "GP0 -> RX")
        wire(draw, mp3["TX"], pico["GP1"], WIRE_RX, "GP1 -> TX")
        draw_speaker(draw, (1250, 965))
        wire(draw, (1250, 800), (1250, 905), WIRE_AUDIO, "audio")

    if step in (1, 2, 3, 4, 5, 6):
        highlights = {
            1: (480, 448, 1020, 552),
            2: (100, 160, 1300, 220),
            3: (150, 250, 330, 400),
            4: (1070, 250, 1250, 400),
            5: (170, 660, 320, 800),
            6: (1070, 660, 1250, 800),
        }
        dashed_rect(draw, highlights[step])

    image.save(destination)


def main() -> None:
    parser = argparse.ArgumentParser(description="Draw breadboard assembly pictures.")
    parser.add_argument(
        "output_dir",
        nargs="?",
        default="schematics/images",
        type=Path,
        help="directory for the generated PNG files",
    )
    args = parser.parse_args()
    args.output_dir.mkdir(parents=True, exist_ok=True)
    for step in range(1, 8):
        destination = args.output_dir / f"step-{step:02d}.png"
        render_step(step, destination)
        print(f"wrote {destination}")


if __name__ == "__main__":
    main()
