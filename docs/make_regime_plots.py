#!/usr/bin/env python3
"""Render per-task regime plots from docs/regime_benchmarks.json.

Run with:
  env -u PYTHONPATH uv run --isolated --no-project --python 3.12 --with matplotlib \
    python3 docs/make_regime_plots.py

Produces docs/regime_benchmarks.svg: one row per task (insert, merge, cardinality, union, sketch),
with a speed panel (left) and, for the estimation tasks, an accuracy panel (right). Each panel plots
the metric against cardinality (log x) and compares the modalities with +/-1 standard-deviation
bands. The register sub-regimes (linear counting, bias corrected, raw) are shaded along the
cardinality axis; the dotted verticals mark the representation transitions.
"""

import json
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
from matplotlib.patches import Patch
from matplotlib.lines import Line2D

REPO = Path(__file__).parent.parent
DATA_FILE = REPO / "docs" / "regime_benchmarks.json"
OUT_FILE = REPO / "docs" / "regime_benchmarks.svg"

with open(DATA_FILE) as f:
    data = json.load(f)

records = data["records"]
dense_start = data["dense_start"]
linear_to_bias = data["linear_to_bias"]
bias_to_raw = data["bias_to_raw"]
value_list_capacity = data["value_list_capacity"]

# (task, has_accuracy). insert and merge are exact operations with no estimation error.
TASKS = [("insert", False), ("merge", False), ("cardinality", True), ("union", True), ("sketch", True)]

# Display titles; the sketch reports its grid size (M = N = 2 in this benchmark).
TASK_TITLE = {"sketch": "sketch (M = N = 2)"}

# "registers" is the standard register-based estimate (the one you get without calling `.mle()`),
# which applies linear counting / bias correction / raw by load (the shaded regions). "registers MLE"
# is the maximum-likelihood register estimate.
STYLE = {
    "value list": ("#1b7837", "o"),
    "hash list": ("#2166ac", "s"),
    "registers": ("#b2182b", "^"),
    "registers MLE": ("#762a83", "D"),
    # The all-hash-list MLE joint sketch (corrected inclusion-exclusion); should track the default
    # hash-list sketch, so a divergence here flags a return of the old exact-decomposition degradation.
    "hash list MLE": ("#35978f", "v"),
    # The hybrid is the real auto-switching counter; drawn black on top to trace the actual path.
    "hybrid": ("#111111", "x"),
    # The `.mle()` path on the real auto-switching counter (exact value list, corrected hash list,
    # register MLE once dense): the MLE you actually get as a counter grows.
    "hybrid MLE": ("#e08214", "*"),
}

# The shading is the regime of the `registers` estimate. Because that curve is force-dense
# (`into_hll`) at every cardinality, it is in linear counting for the WHOLE range below the threshold,
# not just the narrow band where a naturally grown counter happens to be dense. So the linear-counting
# region runs from the left edge up to the linear/bias threshold.
REGIME_SHADE = [
    (None, linear_to_bias, "#cdeccd", "linear counting"),
    (linear_to_bias, bias_to_raw, "#fde7d0", "bias corrected"),
    (bias_to_raw, None, "#f7d4d4", "raw"),
]


def series_for(representation):
    return sorted(
        (r for r in records if r["representation"] == representation),
        key=lambda r: r["cardinality"],
    )


VALUE = series_for("value_list")
HASH = series_for("hash_list")
DENSE = series_for("dense")
HYBRID = series_for("hybrid")


def plot_series(ax, recs, getter, label, floor):
    color, marker = STYLE[label]
    xs, ys, es = [], [], []
    for r in recs:
        v = getter(r)
        if v is None:
            continue
        xs.append(r["cardinality"])
        ys.append(max(v[0], floor))
        es.append(v[1])
    if not xs:
        return
    lo = [max(y - e, floor) for y, e in zip(ys, es)]
    hi = [y + e for y, e in zip(ys, es)]
    ax.plot(xs, ys, marker=marker, ms=3.5, lw=1.4, color=color, label=label)
    ax.fill_between(xs, lo, hi, color=color, alpha=0.18, linewidth=0)


def shade_regimes(ax):
    xmin, xmax = ax.get_xlim()
    for lo, hi, color, _ in REGIME_SHADE:
        lo = lo if lo is not None else xmin
        hi = hi if hi is not None else xmax
        if hi > lo:
            ax.axvspan(lo, hi, color=color, alpha=0.6, zorder=0)
    # Only the two REPRESENTATION transitions get a vertical: value list -> hash list (at the value
    # list capacity) and hash list -> registers (at saturation). The linear/bias/raw boundaries are
    # correction regimes WITHIN the registers representation and are shown by the shading colors, not
    # by a "representation transition" line.
    for x in [value_list_capacity, dense_start]:
        if x:
            ax.axvline(x, color="#777", ls=":", lw=0.8, zorder=1)


def stat(node):
    return (node["mean"], node["std"]) if node else None


def insert_speed(r):
    return stat(r["insert_ns"])


def merge_speed(r):
    return stat(r["merge_ns"])


def op_speed(op, kind):
    def g(r):
        node = r["operations"].get(op, {}).get(kind)
        return stat(node["speed_ns"]) if node else None

    return g


def op_mre(op, kind):
    def g(r):
        node = r["operations"].get(op, {}).get(kind)
        return (node["mre"]["mean"] * 100.0, node["mre"]["std"] * 100.0) if node else None

    return g


fig, axes = plt.subplots(len(TASKS), 2, figsize=(13, 2.9 * len(TASKS)))
fig.suptitle(
    f"HyperLogLog<Precision{data['precision']}, Bits{data['bits']}> "
    f"({data['num_registers']} registers): speed and accuracy by regime\n"
    f"shading = the regime of the registers estimate (sketch is M=N=2, register and hash-list operands)  |  "
    f"bands = +/-1 std over {data['reps']} timing runs (speed) and {data['trials']} trials (accuracy)",
    fontsize=11, y=0.997,
)

# Figure-level legend for the shaded regime bands and the transition verticals.
shade_handles = [Patch(facecolor=c, edgecolor="none", alpha=0.6, label=n) for _, _, c, n in REGIME_SHADE]
shade_handles.append(Line2D([0], [0], color="#777", ls=":", lw=1.0, label="representation transition"))
fig.legend(handles=shade_handles, loc="upper center", ncol=4, fontsize=9,
           bbox_to_anchor=(0.5, 0.965), frameon=False)

SPEED_FLOOR = 1.0  # ns
MRE_FLOOR = 1e-3   # 0.001%, so the exact value-list curve is visible on the log axis.

for row, (task, has_acc) in enumerate(TASKS):
    ax_speed, ax_acc = axes[row]

    if task in ("insert", "merge"):
        getter = insert_speed if task == "insert" else merge_speed
        plot_series(ax_speed, VALUE, getter, "value list", SPEED_FLOOR)
        plot_series(ax_speed, HASH, getter, "hash list", SPEED_FLOOR)
        plot_series(ax_speed, DENSE, getter, "registers", SPEED_FLOOR)
        plot_series(ax_speed, HYBRID, getter, "hybrid", SPEED_FLOOR)
        ax_speed.set_ylabel(f"{task}\nns / call")
        ax_speed.set_title(
            f"{TASK_TITLE.get(task, task)} speed" + (" (per element)" if task == "insert" else "")
        )
        ax_acc.axis("off")
        ax_acc.text(0.5, 0.5, f"{task} is an exact operation\n(no accuracy panel)",
                    ha="center", va="center", fontsize=11, color="#666",
                    transform=ax_acc.transAxes)
    else:
        plot_series(ax_speed, VALUE, op_speed(task, "default"), "value list", SPEED_FLOOR)
        plot_series(ax_speed, HASH, op_speed(task, "default"), "hash list", SPEED_FLOOR)
        plot_series(ax_speed, DENSE, op_speed(task, "default"), "registers", SPEED_FLOOR)
        plot_series(ax_speed, DENSE, op_speed(task, "mle"), "registers MLE", SPEED_FLOOR)
        # Only the sketch records an MLE node for hash-list operands; for the scalar tasks this is None
        # and plots nothing.
        plot_series(ax_speed, HASH, op_speed(task, "mle"), "hash list MLE", SPEED_FLOOR)
        plot_series(ax_speed, HYBRID, op_speed(task, "default"), "hybrid", SPEED_FLOOR)
        plot_series(ax_speed, HYBRID, op_speed(task, "mle"), "hybrid MLE", SPEED_FLOOR)
        ax_speed.set_ylabel(f"{task}\nns / call")
        ax_speed.set_title(f"{TASK_TITLE.get(task, task)}: speed")

        plot_series(ax_acc, VALUE, op_mre(task, "default"), "value list", MRE_FLOOR)
        plot_series(ax_acc, HASH, op_mre(task, "default"), "hash list", MRE_FLOOR)
        plot_series(ax_acc, DENSE, op_mre(task, "default"), "registers", MRE_FLOOR)
        plot_series(ax_acc, DENSE, op_mre(task, "mle"), "registers MLE", MRE_FLOOR)
        plot_series(ax_acc, HASH, op_mre(task, "mle"), "hash list MLE", MRE_FLOOR)
        plot_series(ax_acc, HYBRID, op_mre(task, "default"), "hybrid", MRE_FLOOR)
        plot_series(ax_acc, HYBRID, op_mre(task, "mle"), "hybrid MLE", MRE_FLOOR)
        ax_acc.set_ylabel("mean relative error %")
        ax_acc.set_title(f"{TASK_TITLE.get(task, task)}: accuracy")

    for ax in (ax_speed, ax_acc):
        if ax.has_data():
            ax.set_xscale("log")
            ax.set_yscale("log")
            shade_regimes(ax)
            ax.grid(True, which="major", ls="-", lw=0.3, alpha=0.4)
            ax.set_xlabel("cardinality")
            ax.legend(fontsize=7, loc="best", framealpha=0.85)

plt.tight_layout(rect=[0, 0, 1, 0.95])
fig.savefig(OUT_FILE, format="svg", bbox_inches="tight")
print(f"Written: {OUT_FILE}")
