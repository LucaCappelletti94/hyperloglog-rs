#!/usr/bin/env python3
"""Render docs/value_type_insert.svg from docs/value_type_insert.json.

Run with:
  env -u PYTHONPATH uv run --isolated --no-project --python 3.12 --with matplotlib \
    python3 docs/make_value_type_plot.py

One panel: amortized insertion time (ns per element) versus cardinality, for the value list and the
hash list, with u64 / u32 / u16 values. Shows that the value-list insert cost scales with the value
width (smaller integers pack tighter and splice faster, and survive to higher cardinality), while the
hash list is nearly value-type independent.
"""

import json
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt

REPO = Path(__file__).parent.parent
DATA = REPO / "docs" / "value_type_insert.json"
OUT = REPO / "docs" / "value_type_insert.svg"

with open(DATA) as f:
    data = json.load(f)

TYPE_COLOR = {"u64": "#d62728", "u32": "#1f77b4", "u16": "#2ca02c"}
REPR_STYLE = {"value_list": ("-", "o", "value list"), "hash_list": ("--", "s", "hash list")}

fig, ax = plt.subplots(figsize=(10, 6.2))

caps = {}
for s in data["series"]:
    rows = s["rows"]
    if not rows:
        continue
    xs = [r["cardinality"] for r in rows]
    ys = [max(r["mean"], 1.0) for r in rows]
    es = [r["std"] for r in rows]
    color = TYPE_COLOR[s["value_type"]]
    ls, marker, repr_label = REPR_STYLE[s["representation"]]
    ax.plot(xs, ys, ls=ls, marker=marker, ms=4, lw=1.5, color=color,
            label=f"{repr_label}, {s['value_type']}")
    lo = [max(y - e, 1.0) for y, e in zip(ys, es)]
    hi = [y + e for y, e in zip(ys, es)]
    ax.fill_between(xs, lo, hi, color=color, alpha=0.15, linewidth=0)
    if s["representation"] == "value_list":
        caps[s["value_type"]] = s["capacity"]

# Mark each value-list capacity (where the exact representation gives out).
for vtype, cap in caps.items():
    ax.axvline(cap, color=TYPE_COLOR[vtype], ls=":", lw=0.9, alpha=0.7)
    ax.text(cap, ax.get_ylim()[1], f" {vtype} cap\n {cap}", color=TYPE_COLOR[vtype],
            fontsize=7, va="top", ha="left")

ax.set_xscale("log")
ax.set_yscale("log")
ax.set_xlabel("cardinality")
ax.set_ylabel("amortized insert time (ns / element)")
ax.set_title(
    f"HyperLogLog<Precision{data['precision']}, Bits{data['bits']}>: insertion time by value width\n"
    f"value list scales with the value magnitude; hash list hashes to a fixed width "
    f"(bands = +/-1 std over {data['reps']} runs)",
    fontsize=11,
)
ax.grid(True, which="major", ls="-", lw=0.3, alpha=0.4)
ax.legend(fontsize=8, ncol=2, loc="upper left")

fig.tight_layout()
fig.savefig(OUT, format="svg", bbox_inches="tight")
print(f"Written: {OUT}")
