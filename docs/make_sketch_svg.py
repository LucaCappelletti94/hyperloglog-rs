#!/usr/bin/env python3
"""Generate docs/sketch_benchmark.svg from docs/sketch_benchmark.json.

Run with: uv run python3 docs/make_sketch_svg.py

The joint hypersphere sketch benchmark on random spheres: two nested chains of HLL counters, each
layer adding a fixed number of fresh uniform u32 values. Left panel is the whole-decomposition error
of the pairwise (HLL++ plus inclusion-exclusion) sketch vs the joint MLE as M=N grows; right panel is
the wall-clock time of the exact (HashSet), pairwise, and MLE decompositions.

All text is ASCII; the output is valid XML.
"""

import json
import math
import xml.etree.ElementTree as ET
from pathlib import Path

REPO = Path(__file__).parent.parent
DATA = REPO / "docs" / "sketch_benchmark.json"
OUT = REPO / "docs" / "sketch_benchmark.svg"

blob = json.load(open(DATA))
rows = sorted([r for r in blob["rows"] if r["M"] == r["N"]], key=lambda r: r["M"])
layer = blob.get("layer", 0)
universe = blob.get("range", 0)
ms_axis = [r["M"] for r in rows] or [1]
m_min, m_max = min(ms_axis), max(ms_axis)

W, H = 1560, 600
COL_EXACT = "#2E7D32"   # exact HashSet baseline
COL_PAIR = "#2E86AB"    # pairwise (HLL++ union, inclusion-exclusion)
COL_UNION2 = "#E8871E"  # Ertl 2-set MLE union, applied repeatedly
COL_MLE = "#D62246"     # generalized joint MLE
TEXT = "#1A1A2E"

svg = ET.Element("svg", xmlns="http://www.w3.org/2000/svg",
                 width=str(W), height=str(H), viewBox=f"0 0 {W} {H}")

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

def line(parent, x1, y1, x2, y2, stroke, width=2, dash=None):
    el = tag(parent, "line", x1=x1, y1=y1, x2=x2, y2=y2, stroke=stroke, stroke_width=width)
    if dash:
        el.set("stroke-dasharray", dash)

def polyline(parent, pts, stroke, width=2.5, dash=None):
    s = " ".join(f"{x:.1f},{y:.1f}" for x, y in pts)
    el = tag(parent, "polyline", points=s, fill="none", stroke=stroke, stroke_width=width)
    if dash:
        el.set("stroke-dasharray", dash)

tag(svg, "rect", x=0, y=0, width=W, height=H, fill="#FFFFFF")
text(svg, W // 2, 30, "Joint hypersphere sketch: pairwise HLL++ vs repeated 2-set MLE vs joint MLE (Precision12, Bits6)",
     size=16, weight="bold", fill="#111")
text(svg, W // 2, 50,
     f"Two nested chains of random spheres, {layer:,} fresh values per layer sampled uniformly from 0..{universe:,}.",
     size=12, fill="#555")

PAD_L, PAD_R, PAD_T, PAD_B = 72, 24, 80, 78
GAP = 64
PANEL_W = (W - PAD_L - PAD_R - 2 * GAP) // 3
PANEL_H = H - PAD_T - PAD_B

def panel(x0, title, y_max, y_ticks, log=False, tick_fmt=lambda t: f"{t:.0f}%", y_min=0.0):
    y0, y1 = PAD_T, PAD_T + PANEL_H
    x1 = x0 + PANEL_W
    tag(svg, "rect", x=x0, y=y0, width=PANEL_W, height=PANEL_H, fill="#F8F9FA",
        stroke="#CCC", stroke_width=1, rx=4)
    text(svg, x0 + PANEL_W // 2, y0 - 12, title, size=12, weight="bold", fill="#333")

    def mx(m):
        if m_max == m_min:
            return x0 + PANEL_W / 2
        return x0 + 34 + (m - m_min) / (m_max - m_min) * (PANEL_W - 56)

    def my(v):
        if log:
            lo, hi = math.log10(y_ticks[0]), math.log10(y_max)
            f = (math.log10(max(v, y_ticks[0])) - lo) / (hi - lo)
        else:
            f = (v - y_min) / (y_max - y_min)
        return y1 - f * (PANEL_H - 22) - 12

    for t in y_ticks:
        yy = my(t)
        line(svg, x0, yy, x1, yy, "#E3E6EA", 1)
        text(svg, x0 - 8, yy + 4, tick_fmt(t), size=10, fill="#777", anchor="end")
    for m in range(m_min, m_max + 1):
        text(svg, mx(m), y1 + 18, str(m), size=10, fill="#777")
    text(svg, x0 + PANEL_W // 2, y1 + 38, "M = N (nested layers)", size=11, fill="#555")
    return mx, my

def series(mx, my, key, color, scale=1.0, dash=None):
    coords = [(mx(r["M"]), my(r[key] * scale)) for r in rows]
    polyline(svg, coords, color, dash=dash)
    for x, y in coords:
        tag(svg, "circle", cx=f"{x:.1f}", cy=f"{y:.1f}", r=3, fill=color)

def band(mx, my, key, key_std, color, scale=1.0, opacity=0.16):
    # Shaded +/-1 std region: mean+std along the top, mean-std (clamped at 0) back along the bottom.
    top = [(mx(r["M"]), my((r[key] + r[key_std]) * scale)) for r in rows]
    bot = [(mx(r["M"]), my(max(0.0, r[key] - r[key_std]) * scale)) for r in rows]
    pts = " ".join(f"{x:.1f},{y:.1f}" for x, y in top + bot[::-1])
    tag(svg, "polygon", points=pts, fill=color, fill_opacity=opacity, stroke="none")

# ---- Panel A: overlap-grid accuracy (linear %), the joint sketch's actual job ----
# Each series is a mean line over `seeds` independent seed pairs with a +/-1 std shaded band.
err_keys = ("overlap_err_pairwise", "overlap_err_union2", "overlap_err_mle")
seeds = rows[0].get("seeds", 1)
err_max = max([(r[k] + r[k + "_std"]) * 100 for r in rows for k in err_keys] + [1.0])
y_max_a = math.ceil(err_max / 10) * 10
step = max(5, math.ceil(y_max_a / 5 / 5) * 5)
ticks_a = list(range(0, y_max_a + 1, step))
mxa, mya = panel(
    PAD_L, f"Overlap-grid error, mean +/-1 std over {seeds} seeds (lower is better)", y_max_a, ticks_a
)
for key, col in zip(err_keys, (COL_PAIR, COL_UNION2, COL_MLE)):
    band(mxa, mya, key, key + "_std", col, 100.0)
series(mxa, mya, "overlap_err_pairwise", COL_PAIR, 100.0)
series(mxa, mya, "overlap_err_union2", COL_UNION2, 100.0)
series(mxa, mya, "overlap_err_mle", COL_MLE, 100.0)

# ---- Panel B: paired joint-MLE advantage over 2-set MLE (the proper, significance comparison) ----
# Per-seed difference (2set - joint), so the common seed variance cancels; bars are +/-1 standard
# error of the mean. Points above the zero line with bars clear of zero = significantly better.
pv = [r["paired_2set_minus_joint"] * 100 for r in rows]
ps = [r["paired_2set_minus_joint_sem"] * 100 for r in rows]
lo = min([0.0] + [v - s for v, s in zip(pv, ps)])
hi = max([v + s for v, s in zip(pv, ps)] + [0.5])
y_min_c, y_max_c = math.floor(lo - 0.3), math.ceil(hi + 0.3)
ticks_c = list(range(y_min_c, y_max_c + 1))
x0c = PAD_L + PANEL_W + GAP
mxc, myc = panel(x0c, "Joint MLE advantage over 2-set MLE (paired, +/-1 sem)", y_max_c, ticks_c,
                 tick_fmt=lambda t: f"{t:.0f}%", y_min=y_min_c)
line(svg, x0c, myc(0.0), x0c + PANEL_W, myc(0.0), "#888", 1.5)  # emphasized zero baseline
polyline(svg, [(mxc(r["M"]), myc(v)) for r, v in zip(rows, pv)], COL_MLE, width=2.0)
for r, v, s in zip(rows, pv, ps):
    x = mxc(r["M"])
    line(svg, x, myc(v - s), x, myc(v + s), COL_MLE, 2)
    line(svg, x - 4, myc(v + s), x + 4, myc(v + s), COL_MLE, 2)
    line(svg, x - 4, myc(v - s), x + 4, myc(v - s), COL_MLE, 2)
    tag(svg, "circle", cx=f"{x:.1f}", cy=f"{myc(v):.1f}", r=3, fill=COL_MLE)

# ---- Panel C: time (log ms): exact, pairwise (HLL++), 2-set MLE repeated, joint MLE ----
t_keys = ("time_exact_ms", "time_pairwise_ms", "time_union2_ms", "time_mle_ms")
t_vals = [r[k] for r in rows for k in t_keys]
t_max_real = max(t_vals + [0.01])
ticks_b = [t for t in [0.001, 0.01, 0.1, 1, 10, 100, 1000, 10000] if t <= 10 ** math.ceil(math.log10(t_max_real))]
y_max_b = ticks_b[-1]

def fmt_ms(t):
    return f"{t*1000:.0f}us" if t < 1 else f"{t:.0f}ms"

x0b = PAD_L + 2 * (PANEL_W + GAP)
mxb, myb = panel(x0b, "Wall-clock per decomposition (log scale)", y_max_b, ticks_b, log=True, tick_fmt=fmt_ms)
series(mxb, myb, "time_exact_ms", COL_EXACT)
series(mxb, myb, "time_pairwise_ms", COL_PAIR)
series(mxb, myb, "time_union2_ms", COL_UNION2)
series(mxb, myb, "time_mle_ms", COL_MLE)

# ---- Legend ----
ly = H - 30
items = [
    (COL_EXACT, "exact (HashSet)"),
    (COL_PAIR, "pairwise (HLL++)"),
    (COL_UNION2, "2-set MLE (repeated)"),
    (COL_MLE, "joint MLE"),
]
lx = PAD_L
for color, label in items:
    line(svg, lx, ly, lx + 26, ly, color, 3)
    text(svg, lx + 32, ly + 4, label, size=11, anchor="start", fill="#333")
    lx += 32 + len(label) * 7 + 24

text(svg, W // 2, H - 8, "Measured on AMD Ryzen Threadripper PRO 5975WX, release build.",
     size=10, fill="#999")

ET.indent(ET.ElementTree(svg), space="  ")
svg_text = '<?xml version="1.0" encoding="UTF-8"?>\n' + ET.tostring(svg, encoding="unicode")
OUT.write_text(svg_text, encoding="utf-8")
ET.fromstring(svg_text)
print(f"Written {OUT} ({len(svg_text)} bytes), points={len(rows)}")
