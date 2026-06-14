#!/usr/bin/env python3
"""Generate docs/architecture.svg from docs/regime_benchmarks.json.

Run with: uv run python3 docs/make_architecture_svg.py
The script produces a single, self-contained SVG that shows:
  - The three-representation ladder (exact, hash-list, dense) as labeled boxes
  - The switch points annotated with the measured cardinality thresholds
  - Speed and quality bar charts for each regime, positioned near the relevant layer

All text is ASCII only.  The output is valid XML (verified with xml.etree).
"""

import json
import math
import xml.etree.ElementTree as ET
from pathlib import Path

REPO = Path(__file__).parent.parent
DATA_FILE = REPO / "docs" / "regime_benchmarks.json"
OUT_FILE = REPO / "docs" / "architecture.svg"

# ---------------------------------------------------------------------------
# Load data
# ---------------------------------------------------------------------------
with open(DATA_FILE) as f:
    data = json.load(f)

exact_end = data["switch_exact_to_hash_list"]
dense_start = data["switch_hash_list_to_dense"]
rows = data["rows"]

def rows_for(regime):
    return [r for r in rows if r["regime"] == regime]

exact_rows = rows_for("exact")
hash_rows = rows_for("hash_list")
dense_rows = rows_for("dense")

def mean(vals):
    return sum(vals) / len(vals) if vals else 0.0

def median(vals):
    """Median, which is robust to the transitional outlier right at the dense boundary (a counter
    that has only just converted shows an elevated union error)."""
    if not vals:
        return 0.0
    s = sorted(vals)
    n = len(s)
    return s[n // 2] if n % 2 else 0.5 * (s[n // 2 - 1] + s[n // 2])

def fmt_ns(ns):
    """Return a human-readable speed string."""
    if ns < 1_000:
        return f"{ns:.0f} ns"
    if ns < 1_000_000:
        return f"{ns/1_000:.1f} us"
    if ns < 1_000_000_000:
        return f"{ns/1_000_000:.1f} ms"
    return f"{ns/1_000_000_000:.1f} s"

def fmt_pct(v):
    return f"{v*100:.2f}%"

# Aggregate per-regime averages (excluding card=1 for exact merge outlier at card=1 it's fine).
regimes_data = {}
for name, rlist in [("exact", exact_rows), ("hash_list", hash_rows), ("dense", dense_rows)]:
    if not rlist:
        continue
    regimes_data[name] = {
        "insert_ns":    median([r["insert_ns"] for r in rlist]),
        "card_ns":      median([r["est_card_ns"] for r in rlist]),
        "union_ns":     median([r["est_union_ns"] for r in rlist]),
        "merge_ns":     median([r["merge_ns"] for r in rlist]),
        "mle_card_ns":  median([r["mle_card_ns"] for r in rlist]),
        "mle_union_ns": median([r["mle_union_ns"] for r in rlist]),
        "jmle_ns":      median([r["joint_mle_ns"] for r in rlist]),
        "def_card_mre": median([r["default_card_mre"] for r in rlist]),
        "def_union_mre":median([r["default_union_mre"] for r in rlist]),
        "mle_union_mre":median([r["mle_union_mre"] for r in rlist]),
    }

# ---------------------------------------------------------------------------
# SVG layout constants
# ---------------------------------------------------------------------------
W = 1100   # total width
H = 780    # total height

BOX_W = 220
BOX_H = 70
BOX_Y = [100, 300, 500]          # y-top of each regime box
BOX_X = 40                        # left edge of regime boxes
ARROW_MID_X = BOX_X + BOX_W // 2

CHART_X = BOX_X + BOX_W + 30    # left edge of bar charts
CHART_W = 780                     # total chart area width (two panels side by side)
CHART_H = 185                     # height of each chart group

REGIME_NAMES = ["exact", "hash_list", "dense"]
REGIME_LABELS = ["Exact-values\n(feature exact)", "Hash List\n(sorted composite hashes)", "Dense Registers\n(classic HyperLogLog)"]
COLORS = {"exact": "#4E8098", "hash_list": "#90C2E7", "dense": "#C4EAE1"}
BAR_COLOR_DEF = "#2E86AB"
BAR_COLOR_MLE = "#A23B72"
BAR_COLOR_MERGE = "#F18F01"
TEXT_COLOR = "#1A1A2E"

# ---------------------------------------------------------------------------
# SVG builder helpers
# ---------------------------------------------------------------------------
def tag(parent, name, text=None, **attrs):
    """Append a child element and return it."""
    el = ET.SubElement(parent, name)
    for k, v in attrs.items():
        el.set(k.replace("_", "-"), str(v))
    if text is not None:
        el.text = str(text)
    return el

def rect(parent, x, y, w, h, fill, stroke="#333", stroke_width=1.5, rx=6):
    tag(parent, "rect", x=x, y=y, width=w, height=h,
        fill=fill, stroke=stroke, stroke_width=stroke_width, rx=rx)

def text(parent, x, y, content, font_size=13, fill=TEXT_COLOR, anchor="middle",
         weight="normal", family="monospace"):
    el = tag(parent, "text", content,
             x=x, y=y, font_family=family, font_size=font_size,
             fill=fill, text_anchor=anchor, font_weight=weight)
    return el

def line(parent, x1, y1, x2, y2, stroke="#555", stroke_width=2, dash=None):
    el = tag(parent, "line", x1=x1, y1=y1, x2=x2, y2=y2,
             stroke=stroke, stroke_width=stroke_width)
    if dash:
        el.set("stroke-dasharray", dash)
    return el

def arrow(parent, x1, y1, x2, y2, color="#555"):
    """Draw a line with an arrowhead at (x2,y2)."""
    line(parent, x1, y1, x2, y2, stroke=color, stroke_width=2)
    # Simple chevron arrowhead
    dx = x2 - x1
    dy = y2 - y1
    length = math.hypot(dx, dy)
    if length < 1:
        return
    ux, uy = dx / length, dy / length
    px, py = -uy, ux
    size = 10
    pts = f"{x2},{y2} {x2-ux*size+px*4},{y2-uy*size+py*4} {x2-ux*size-px*4},{y2-uy*size-py*4}"
    tag(parent, "polygon", points=pts, fill=color)

# ---------------------------------------------------------------------------
# Bar chart helper
# ---------------------------------------------------------------------------
def speed_bar_chart(parent, x, y, w, h, values, title):
    """Draw a horizontal bar chart on a log10 scale (the values span 1 ns to ~200 ms).

    values: list of (label, ns_value, color) triples
    """
    margin_left = 92
    label_room = 48
    bar_area_w = w - margin_left - label_room
    bar_h = 16
    gap = 5

    axis_max_ns = max((v for _, v, _ in values), default=1.0)
    log_max = math.log10(max(axis_max_ns, 10.0))

    # Background panel.
    tag(parent, "rect", x=x, y=y, width=w, height=h,
        fill="#F8F9FA", stroke="#CCC", stroke_width=1, rx=4)

    # Title.
    text(parent, x + w // 2, y + 16, title,
         font_size=12, weight="bold", fill="#333", family="sans-serif")

    by = y + 28
    for label, val, color in values:
        log_v = math.log10(max(val, 1.0))
        bw = int(bar_area_w * log_v / log_max) if log_max > 0 else 2
        bw = max(2, min(bw, bar_area_w))
        tag(parent, "rect", x=x + margin_left, y=by, width=bw, height=bar_h,
            fill=color, rx=2)
        text(parent, x + margin_left - 4, by + bar_h - 4, label,
             font_size=10, fill="#333", anchor="end", family="sans-serif")
        text(parent, x + margin_left + bw + 4, by + bar_h - 4, fmt_ns(val),
             font_size=10, fill="#444", anchor="start", family="sans-serif")
        by += bar_h + gap


def quality_bar_chart(parent, x, y, w, h, values, title):
    """Draw quality (MRE) bar chart. values: list of (label, mre, color)."""
    max_val = max(v for _, v, _ in values) if values else 0.01
    if max_val == 0:
        max_val = 0.01
    max_val = max(max_val, 0.005)  # ensure visible bars

    margin_left = 90
    bar_area_w = w - margin_left - 60
    bar_h = 16
    gap = 5

    tag(parent, "rect", x=x, y=y, width=w, height=h,
        fill="#F8F9FA", stroke="#CCC", stroke_width=1, rx=4)

    text(parent, x + w // 2, y + 16, title,
         font_size=12, weight="bold", fill="#333", family="sans-serif")

    by = y + 26
    for label, val, color in values:
        bw = int(bar_area_w * val / max_val)
        bw = max(bw, 2)
        tag(parent, "rect", x=x + margin_left, y=by, width=bw, height=bar_h,
            fill=color, rx=2)
        text(parent, x + margin_left - 4, by + bar_h - 4, label,
             font_size=10, fill="#333", anchor="end", family="sans-serif")
        text(parent, x + margin_left + bw + 4, by + bar_h - 4, fmt_pct(val),
             font_size=10, fill="#444", anchor="start", family="sans-serif")
        by += bar_h + gap

# ---------------------------------------------------------------------------
# Build SVG
# ---------------------------------------------------------------------------
svg = ET.Element("svg",
    xmlns="http://www.w3.org/2000/svg",
    width=str(W),
    height=str(H),
    viewBox=f"0 0 {W} {H}")

# Background
tag(svg, "rect", x=0, y=0, width=W, height=H, fill="#FFFFFF")

# Title
text(svg, W // 2, 40, "HyperLogLog-rs: Three-Representation Ladder (Precision12, Bits6)",
     font_size=17, weight="bold", family="sans-serif", fill="#111")
text(svg, W // 2, 60,
     f"2^12 = 4096 registers  |  exact-values -> hash-list at cardinality ~{exact_end}  |  hash-list -> dense at ~{dense_start}",
     font_size=12, family="sans-serif", fill="#555")

# ---------------------------------------------------------------------------
# Regime boxes
# ---------------------------------------------------------------------------
regime_info = [
    ("exact", "Exact-values mode",
     ["Sorted, gap-coded, gamma-packed values", f"Active for cardinality 0 .. {exact_end}", "Zero estimation error; exact set ops",
      "(requires 'exact' feature + alloc)"],
     BOX_Y[0]),
    ("hash_list", "Hash-list mode",
     ["Sorted gap-coded composite hashes", f"Active for cardinality ~{exact_end} .. {dense_start}", "Near-exact: MRE < 0.65%  (union < 1.3%)"],
     BOX_Y[1]),
    ("dense", "Dense registers (HyperLogLog)",
     [f"Classic HLL register array (4096 x 6 bits)", f"Active for cardinality {dense_start}+", "Default MRE ~1-1.5%;  MLE union MRE ~1%"],
     BOX_Y[2]),
]

for key, title, lines, by in regime_info:
    color = COLORS[key]
    bx = BOX_X
    bh = 20 + len(lines) * 17 + 12
    rect(svg, bx, by, BOX_W, bh, fill=color, stroke="#2A5F7A", stroke_width=2)
    text(svg, bx + BOX_W // 2, by + 18, title,
         font_size=13, weight="bold", fill="#111", family="sans-serif")
    for i, ln in enumerate(lines):
        text(svg, bx + 8, by + 36 + i * 17, ln,
             font_size=10, fill="#222", anchor="start", family="sans-serif")

# ---------------------------------------------------------------------------
# Arrows with switch-point annotations
# ---------------------------------------------------------------------------
# exact -> hash_list arrow
ax = ARROW_MID_X
y_exact_bottom = BOX_Y[0] + 20 + len(regime_info[0][2]) * 17 + 12
y_hl_top = BOX_Y[1]
arrow(svg, ax, y_exact_bottom + 2, ax, y_hl_top - 2, color="#C0392B")
mid_y = (y_exact_bottom + y_hl_top) // 2
text(svg, ax + 6, mid_y - 4, f"cardinality ~{exact_end}",
     font_size=11, fill="#C0392B", anchor="start", family="sans-serif")
text(svg, ax + 6, mid_y + 10, "exact overflow ->",
     font_size=10, fill="#C0392B", anchor="start", family="sans-serif")
text(svg, ax + 6, mid_y + 22, "hashes inserted",
     font_size=10, fill="#C0392B", anchor="start", family="sans-serif")

# hash_list -> dense arrow
y_hl_bottom = BOX_Y[1] + 20 + len(regime_info[1][2]) * 17 + 12
y_dense_top = BOX_Y[2]
arrow(svg, ax, y_hl_bottom + 2, ax, y_dense_top - 2, color="#8E44AD")
mid_y2 = (y_hl_bottom + y_dense_top) // 2
text(svg, ax + 6, mid_y2 - 4, f"cardinality ~{dense_start}",
     font_size=11, fill="#8E44AD", anchor="start", family="sans-serif")
text(svg, ax + 6, mid_y2 + 10, "hash list saturates ->",
     font_size=10, fill="#8E44AD", anchor="start", family="sans-serif")
text(svg, ax + 6, mid_y2 + 22, "register array",
     font_size=10, fill="#8E44AD", anchor="start", family="sans-serif")

# ---------------------------------------------------------------------------
# Bar charts per regime
# ---------------------------------------------------------------------------
# Chart parameters chosen so they fit vertically (each regime gets ~130px).
chart_y_positions = [BOX_Y[0], BOX_Y[1], BOX_Y[2]]

# For each regime we show two charts side by side:
#   left: speed bar chart (insert, card, union, merge, mle_union, jmle)
#   right: quality bar chart (def_card, def_union, mle_union)
HALF_W = CHART_W // 2 - 8

for i, key in enumerate(["exact", "hash_list", "dense"]):
    if key not in regimes_data:
        continue
    rd = regimes_data[key]
    cy = chart_y_positions[i]

    # Speed chart (log scale, so the 1 ns to 200 ms range is all visible).
    speed_vals = [
        ("insert",    rd["insert_ns"],    "#5B8DB8"),
        ("card(def)", rd["card_ns"],      BAR_COLOR_DEF),
        ("union(def)",rd["union_ns"],     BAR_COLOR_DEF),
        ("merge",     rd["merge_ns"],     BAR_COLOR_MERGE),
        ("card(MLE)", rd["mle_card_ns"],  BAR_COLOR_MLE),
        ("union(MLE)",rd["mle_union_ns"], BAR_COLOR_MLE),
        ("jMLE",      rd["jmle_ns"],      "#D62246"),
    ]

    speed_bar_chart(svg, CHART_X, cy, HALF_W, CHART_H,
                    speed_vals, f"{key}: speed per call (log scale)")

    # Quality chart.
    if key == "exact":
        qual_vals = [("def card", 0.0, BAR_COLOR_DEF),
                     ("def union", 0.0, BAR_COLOR_DEF),
                     ("MLE union", 0.0, BAR_COLOR_MLE)]
        qual_note = "(exact: all errors = 0.0%)"
    else:
        qual_vals = [
            ("def card",  rd["def_card_mre"],  BAR_COLOR_DEF),
            ("def union", rd["def_union_mre"],  BAR_COLOR_DEF),
            ("MLE union", rd["mle_union_mre"],  BAR_COLOR_MLE),
        ]
        qual_note = ""

    quality_bar_chart(svg, CHART_X + HALF_W + 10, cy, HALF_W, CHART_H,
                      qual_vals, f"{key}: quality (MRE)")
    if qual_note:
        text(svg, CHART_X + HALF_W + 10 + HALF_W // 2, cy + CHART_H - 8, qual_note,
             font_size=10, fill="#555", family="sans-serif")

# ---------------------------------------------------------------------------
# Legend
# ---------------------------------------------------------------------------
lx = BOX_X
ly = H - 64
text(svg, lx, ly, "Legend:", font_size=12, weight="bold", anchor="start", family="sans-serif")
legend_items = [
    (BAR_COLOR_DEF, "Default estimator (HLL++)"),
    (BAR_COLOR_MLE, "MLE estimator (Ertl)"),
    (BAR_COLOR_MERGE, "Merge (|)"),
    ("#D62246",     "Joint MLE (JointSketch::estimate)"),
]
lxi = lx
for color, label in legend_items:
    tag(svg, "rect", x=lxi, y=ly + 10, width=14, height=14, fill=color, rx=2)
    text(svg, lxi + 18, ly + 22, label, font_size=11, anchor="start", family="sans-serif")
    lxi += len(label) * 7 + 30

# ---------------------------------------------------------------------------
# Footer
# ---------------------------------------------------------------------------
text(svg, W // 2, H - 10, f"Measured on AMD Ryzen Threadripper PRO 5975WX | Release build | 100 quality trials per cardinality",
     font_size=10, fill="#999", family="sans-serif")

# ---------------------------------------------------------------------------
# Serialize and validate
# ---------------------------------------------------------------------------
tree = ET.ElementTree(svg)
ET.indent(tree, space="  ")
svg_bytes = ET.tostring(svg, encoding="unicode", xml_declaration=False)

# Prepend XML declaration.
svg_text = '<?xml version="1.0" encoding="UTF-8"?>\n' + svg_bytes

OUT_FILE.write_text(svg_text, encoding="utf-8")
print(f"Written: {OUT_FILE}")
print(f"  Size: {len(svg_text):,} bytes")

# Validate well-formedness.
try:
    ET.fromstring(svg_text)
    print("  XML validation: PASSED (well-formed)")
except ET.ParseError as e:
    print(f"  XML validation: FAILED - {e}")
    raise

# Sanity-check dimensions.
root = ET.fromstring(svg_text)
w_attr = root.get("width")
h_attr = root.get("height")
print(f"  Dimensions: {w_attr} x {h_attr}")
print("Done.")
