#!/usr/bin/env python3
"""Generate docs/sketch_benchmark_normalized.svg from docs/sketch_benchmark.json.

Run with: uv run python3 docs/make_sketch_normalized_svg.py

Companion to make_sketch_svg.py: plots the two normalized overlap-grid error metrics (Phase 3), each
a mean line over `seeds` seed pairs with a +/-1 std band. Left panel is metric (a), per-cell
|est - exact| / exact shell maximum (ground-truth denominators); right panel is metric (b), each
side self-normalized (est.normalize() vs exact.normalize()). They coincide when the marginals are
estimated accurately. All text is ASCII; the output is valid XML.
"""

import json
import math
import xml.etree.ElementTree as ET
from pathlib import Path

REPO = Path(__file__).parent.parent
DATA = REPO / "docs" / "sketch_benchmark.json"
OUT = REPO / "docs" / "sketch_benchmark_normalized.svg"

blob = json.load(open(DATA))
rows = sorted([r for r in blob["rows"] if r["M"] == r["N"]], key=lambda r: r["M"])
seeds = rows[0].get("seeds", 1)
ms_axis = [r["M"] for r in rows] or [1]
m_min, m_max = min(ms_axis), max(ms_axis)

W, H = 1080, 600
COL_PAIR = "#2E86AB"
COL_UNION2 = "#E8871E"
COL_MLE = "#D62246"
TEXT = "#1A1A2E"

svg = ET.Element("svg", xmlns="http://www.w3.org/2000/svg", width=str(W), height=str(H),
                 viewBox=f"0 0 {W} {H}")

def tag(parent, name, text=None, **attrs):
    el = ET.SubElement(parent, name)
    for k, v in attrs.items():
        el.set(k.replace("_", "-"), str(v))
    if text is not None:
        el.text = str(text)
    return el

def text(parent, x, y, content, size=13, fill=TEXT, anchor="middle", weight="normal"):
    tag(parent, "text", content, x=x, y=y, font_family="sans-serif", font_size=size,
        fill=fill, text_anchor=anchor, font_weight=weight)

def line(parent, x1, y1, x2, y2, stroke, width=2):
    tag(parent, "line", x1=x1, y1=y1, x2=x2, y2=y2, stroke=stroke, stroke_width=width)

tag(svg, "rect", x=0, y=0, width=W, height=H, fill="#FFFFFF")
text(svg, W // 2, 30, "Joint hypersphere sketch: normalized overlap-grid error (Precision12, Bits6)",
     size=16, weight="bold", fill="#111")
text(svg, W // 2, 50,
     f"Random spheres, {blob.get('layer', 0):,} per layer from 0..{blob.get('range', 0):,}; "
     f"mean +/-1 std over {seeds} seeds, equal weight per cell.",
     size=12, fill="#555")

PAD_L, PAD_R, PAD_T, PAD_B = 72, 24, 80, 78
PANEL_W = (W - PAD_L - PAD_R - 64) // 2
PANEL_H = H - PAD_T - PAD_B

def panel(x0, title, y_max, ticks):
    y0, y1 = PAD_T, PAD_T + PANEL_H
    x1 = x0 + PANEL_W
    tag(svg, "rect", x=x0, y=y0, width=PANEL_W, height=PANEL_H, fill="#F8F9FA",
        stroke="#CCC", stroke_width=1, rx=4)
    text(svg, x0 + PANEL_W // 2, y0 - 12, title, size=13, weight="bold", fill="#333")

    def mx(m):
        if m_max == m_min:
            return x0 + PANEL_W / 2
        return x0 + 34 + (m - m_min) / (m_max - m_min) * (PANEL_W - 56)

    def my(v):
        return y1 - (v / y_max) * (PANEL_H - 22) - 12

    for t in ticks:
        yy = my(t)
        line(svg, x0, yy, x1, yy, "#E3E6EA", 1)
        text(svg, x0 - 8, yy + 4, f"{t:g}", size=10, fill="#777", anchor="end")
    for m in range(m_min, m_max + 1):
        text(svg, mx(m), y1 + 18, str(m), size=10, fill="#777")
    text(svg, x0 + PANEL_W // 2, y1 + 38, "M = N (nested layers)", size=11, fill="#555")
    return mx, my

def band(mx, my, key, color, opacity=0.16):
    top = [(mx(r["M"]), my(r[key] + r[key + "_std"])) for r in rows]
    bot = [(mx(r["M"]), my(max(0.0, r[key] - r[key + "_std"]))) for r in rows]
    pts = " ".join(f"{x:.1f},{y:.1f}" for x, y in top + bot[::-1])
    tag(svg, "polygon", points=pts, fill=color, fill_opacity=opacity, stroke="none")

def series(mx, my, key, color):
    coords = [(mx(r["M"]), my(r[key])) for r in rows]
    pts = " ".join(f"{x:.1f},{y:.1f}" for x, y in coords)
    tag(svg, "polyline", points=pts, fill="none", stroke=color, stroke_width=2.5)
    for x, y in coords:
        tag(svg, "circle", cx=f"{x:.1f}", cy=f"{y:.1f}", r=3, fill=color)

cols = (("pairwise", COL_PAIR), ("union2", COL_UNION2), ("mle", COL_MLE))

def draw(x0, prefix, title):
    keys = [f"{prefix}_{name}" for name, _ in cols]
    vmax = max([r[k] + r[k + "_std"] for r in rows for k in keys] + [0.001])
    # Round up to a tidy axis max.
    mag = 10 ** math.floor(math.log10(vmax))
    y_max = math.ceil(vmax / mag) * mag
    ticks = [y_max * i / 4 for i in range(5)]
    mx, my = panel(x0, title, y_max, ticks)
    for (name, col), k in zip(cols, keys):
        band(mx, my, k, col)
    for (name, col), k in zip(cols, keys):
        series(mx, my, k, col)

draw(PAD_L, "norm_a", "(a) error / exact shell maximum")
draw(PAD_L + PANEL_W + 64, "norm_b", "(b) self-normalized vs exact-normalized")

# ---- Legend ----
ly = H - 30
lx = PAD_L
for label, col in (("pairwise (HLL++)", COL_PAIR), ("2-set MLE (repeated)", COL_UNION2),
                   ("joint MLE", COL_MLE)):
    line(svg, lx, ly, lx + 26, ly, col, 3)
    text(svg, lx + 32, ly + 4, label, size=11, anchor="start", fill="#333")
    lx += 32 + len(label) * 7 + 24

text(svg, W // 2, H - 8, "Measured on AMD Ryzen Threadripper PRO 5975WX, release build.",
     size=10, fill="#999")

ET.indent(ET.ElementTree(svg), space="  ")
svg_text = '<?xml version="1.0" encoding="UTF-8"?>\n' + ET.tostring(svg, encoding="unicode")
OUT.write_text(svg_text, encoding="utf-8")
ET.fromstring(svg_text)
print(f"Written {OUT} ({len(svg_text)} bytes), points={len(rows)}")
