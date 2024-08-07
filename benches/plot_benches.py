"""Python script to load and plot the benchmarks results."""

from typing import List, Dict
import pandas as pd
import matplotlib.pyplot as plt
from glob import glob


def unit_to_factor(unit: str) -> float:
    """Converts a time unit to a factor."""
    if unit == "ns":
        return 1e-9
    elif unit == "µs":
        return 1e-6
    elif unit == "ms":
        return 1e-3
    elif unit == "s":
        return 1
    else:
        raise ValueError(f"Unknown unit: {unit}")


def is_name_line(line: str) -> bool:
    """Checks if a line is a name line."""
    return " " not in line and "precision" in line and "bits" in line


def normalize_name(name: str) -> str:
    """Normalizes a name."""
    return {
        "hll": "Our HLL++",
        "mle": "Our MLE",
        "multiplicities": "Our Multi",
        "tabacpf": "Tabac's HLL",
        "tabacplusplus": "Tabac's HLL++",
        "sa": "Alec's HLL++",
    }[name]


def parse_name_line(line: str) -> Dict:
    """Parses a name line."""
    # We expect the line to be in the format:
    # hll_cardinality_precision_4_bits_5
    name, task, _, precision, _, bits = line.split("_")
    precision = int(precision)
    bits = int(bits)

    return {
        "name": normalize_name(name),
        "task": task,
        "precision": precision,
        "bits": bits,
        "minimum_memory_usage": (2**precision) * 4,
    }


def is_time_line(line: str) -> bool:
    """Checks if a line is a time line."""
    return line.startswith("time:")


def is_change_line(line: str) -> bool:
    """Checks if a line is a change line."""
    return line.startswith("change:")


def is_performance_comment_line(line: str) -> bool:
    """Checks if a line is a performance comment line."""
    needles = [
        "Performance has improved",
        "Performance has regressed",
        "Change within noise threshold",
        "No change in performance detected."
    ]

    return any(needle in line for needle in needles)


def is_outliers_line(line: str) -> bool:
    """Checks if a line is an outliers line."""
    return "outliers among" in line


def is_outlier_count_line(line: str) -> bool:
    """Checks if a line is an outlier count line."""
    needles = [
        "low severe",
        "low mild",
        "high mild",
        "high severe",
    ]
    return any(needle in line for needle in needles)


def parse_time_line(line: str) -> Dict:
    """Parses a time line."""
    # First we remove the prefix "time:   "
    line = line[5:]

    # Them we strip spaces
    line = line.strip()

    # Next, we remove the brackets
    line = line[1:-1]

    # Finally, we split the line
    parts = line.split()

    # We expect 3 parts, composed by a series of tubles.
    assert len(parts) == 6
    lower_bound = float(parts[0]) * unit_to_factor(parts[1])
    mean = float(parts[2]) * unit_to_factor(parts[3])
    upper_bound = float(parts[4]) * unit_to_factor(parts[5])

    return {
        "lower_bound": lower_bound,
        "mean": mean,
        "upper_bound": upper_bound,
    }


def load_criterion_log(path: str) -> pd.DataFrame:
    """Loads a criterion log file into a pandas DataFrame.

    Implementation details
    ----------------------
    A criterion log looks like the following:

    ```plaintext
    hll_cardinality_precision_4_bits_5
                            time:   [555.03 µs 555.48 µs 556.03 µs]
    Found 9 outliers among 100 measurements (9.00%)
    2 (2.00%) low mild
    1 (1.00%) high mild
    6 (6.00%) high severe

    hll_cardinality_precision_4_bits_6
                            time:   [586.12 µs 588.63 µs 590.62 µs]
    Found 11 outliers among 100 measurements (11.00%)
    7 (7.00%) low severe
    4 (4.00%) low mild
    ```

    """
    with open(path, "r", encoding="utf-8") as f:
        lines = f.readlines()

    rows: List[Dict] = []
    current_line = None

    for line in lines:
        line = line.strip()
        # We skip empty lines
        if (
            not line
            or is_outliers_line(line)
            or is_outlier_count_line(line)
            or is_change_line(line)
            or is_performance_comment_line(line)
        ):
            continue

        try:
            if is_name_line(line):
                if current_line is not None:
                    rows.append(current_line)
                current_line = {}
                current_line.update(parse_name_line(line))
                continue

            if is_time_line(line):
                current_line.update(parse_time_line(line))
                continue
        except ValueError as e:
            raise ValueError(f"Error while parsing line: {line}") from e

        raise NotImplementedError(f"Unknown line: {line}")

    if current_line is not None and current_line:
        rows.append(current_line)

    return pd.DataFrame(rows)


def get_bits_line_style(bits: int) -> Dict:
    """Returns the line style for a given number of bits."""
    return {4: "..", 5: "-.", 6: "-"}[bits]


def get_library_color(library_name: str) -> str:
    """Returns the color for a given normalized library name."""
    return {
        "Our HLL++": "tab:blue",
        "Our MLE": "tab:green",
        "Our Multi": "tab:cyan",
        "Tabac's HLL": "tab:red",
        "Tabac's HLL++": "tab:purple",
        "Alec's HLL++": "tab:orange",
    }[library_name]


def get_library_marker(library_name: str) -> str:
    """Returns the marker for a given normalized library name."""
    return {
        "Our HLL++": ".",
        "Our MLE": ".",
        "Our Multi": ".",
        "Tabac's HLL": "x",
        "Tabac's HLL++": "x",
        "Alec's HLL++": "x",
    }[library_name]


def plot_benchmarks(path: str) -> None:
    """Plots the benchmarks.

    We plot the memory usage on the abscissa and the time on the ordinate,
    with the lower and upper bounds shown both as error bars and as a shaded
    area.
    """
    df = load_criterion_log(path)

    # We expect that the 'task' column is the same for all rows
    assert df["task"].nunique() == 1
    task = df["task"].iloc[0].capitalize()

    fig, ax = plt.subplots(
        ncols=2,  # We plot both in linear and log scale
        figsize=(14, 6),
        dpi=300,
    )

    for name in df["name"].unique():
        for bits in df["bits"].unique():
            mask = (df["name"] == name) & (df["bits"] == bits)
            subdf = df[mask]
            # If the mask is empty, we skip the plot
            if subdf.empty:
                continue
            for scale, axes in zip(["linear", "log"], ax):
                axes.set_xscale("log")
                axes.set_yscale(scale)
                axes.set_xlabel("Memory usage (bytes)")
                axes.set_ylabel("Time (s)")
                axes.set_title(f"{task} - {scale} scale")
                axes.grid(b=True, which="both", axis="both")
                axes.fill_between(
                    subdf["minimum_memory_usage"],
                    subdf["lower_bound"],
                    subdf["upper_bound"],
                    color=get_library_color(name),
                    alpha=0.5,
                )
                axes.errorbar(
                    subdf["minimum_memory_usage"],
                    subdf["mean"],
                    yerr=[
                        subdf["mean"] - subdf["lower_bound"],
                        subdf["upper_bound"] - subdf["mean"],
                    ],
                    linestyle=get_bits_line_style(bits),
                    color=get_library_color(name),
                    label=f"{name} - {bits} bits",
                )
                # We  also plot the points
                axes.scatter(
                    subdf["minimum_memory_usage"],
                    subdf["mean"],
                    marker=get_library_marker(name),
                    color=get_library_color(name),
                )

    # We position the legend outside the plot, on the right
    # of the second axes. In order to avoid duplicating the
    # legend, we only show it on the second axes.
    ax[1].legend(loc="center left", bbox_to_anchor=(1, 0.5))

    fig.tight_layout()

    figure_path = path.replace(".log", ".png")

    plt.savefig(figure_path)


if __name__ == "__main__":
    for log_path in glob("benches/*.log"):
        plot_benchmarks(log_path)
