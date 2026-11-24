#!/usr/bin/env python3
"""Prepares hyprlock: the palette as hyprlang variables, the M3 shapes
behind the clock as PNGs, and a small copy of the wallpaper.

Usage: lock.py [wallpaper]. theme.sh runs it after every palette change;
without a wallpaper it asks awww for the current one.
"""

import math
import os
import re
import subprocess
import sys

HOME = os.path.expanduser("~")
CONFIG = os.path.join(os.environ.get("XDG_CONFIG_HOME") or f"{HOME}/.config", "octopus")
OUT = os.path.join(os.environ.get("XDG_CACHE_HOME") or f"{HOME}/.cache", "octopus", "lock")
SIZE = 512

# (name, lobes, depth, color) — the same polar wave the companion draws.
SHAPES = [
    ("cookie9", 9, 0.055, "primary_container"),
    ("clover4", 4, 0.16, "tertiary_container"),
]


def palette():
    colors = {}
    try:
        with open(os.path.join(CONFIG, "colors.css")) as file:
            for name, value in re.findall(r"@define-color\s+(\w+)\s+#([0-9a-fA-F]{6})", file.read()):
                colors[name] = value.lower()
    except OSError:
        pass
    return colors


def outline(lobes, depth):
    c, radius, steps = SIZE / 2, SIZE / 2, 720
    points = []
    for i in range(steps):
        t = 2 * math.pi * i / steps
        r = radius * (1 + depth * math.cos(lobes * t)) / (1 + depth)
        points.append(f"{c + r * math.cos(t):.2f},{c + r * math.sin(t):.2f}")
    return "M" + " L".join(points) + " Z"


def render(name, svg):
    path = os.path.join(OUT, f"{name}.png")
    subprocess.run(["rsvg-convert", "-o", path], input=svg.encode(), check=False)


def wallpaper(image):
    if not image:
        try:
            query = subprocess.run(["awww", "query"], capture_output=True, text=True).stdout
        except OSError:
            query = ""
        match = re.search(r"image: (.+)", query)
        image = match.group(1).strip() if match else ""
    if image and os.path.isfile(image):
        subprocess.run(["magick", f"{image}[0]", "-resize", "1600x1600>", "-quality", "90",
                        os.path.join(OUT, "wallpaper.jpg")], check=False)


def main():
    os.makedirs(OUT, exist_ok=True)
    colors = palette()
    with open(os.path.join(OUT, "colors.conf"), "w") as file:
        for name, value in colors.items():
            file.write(f"${name} = rgb({value})\n")
    for name, lobes, depth, role in SHAPES:
        fill = "#" + colors.get(role, "888888")
        render(name, f'<svg xmlns="http://www.w3.org/2000/svg" width="{SIZE}" height="{SIZE}">'
                     f'<path d="{outline(lobes, depth)}" fill="{fill}"/></svg>')
    render("blank", '<svg xmlns="http://www.w3.org/2000/svg" width="8" height="8"/>')
    wallpaper(sys.argv[1] if len(sys.argv) > 1 else "")


if __name__ == "__main__":
    main()
