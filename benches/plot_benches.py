"""Python script to load and plot the benchmarks results."""

from typing import List, Dict
from glob import glob
import pandas as pd
import matplotlib.pyplot as plt


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
        "beta6_xxhash64": "Beta6 + Xxhasher",
        "beta6_XxHash64": "Beta6 + Xxhasher",
        "beta6_WyHash": "Beta6 + WyHash",
        "beta6_wyhash": "Beta6 + WyHash",
        "hll4_xxhash64": "HLL4 + Xxhasher",
        "hll4_XxHash64": "HLL4 + Xxhasher",
        "hll4": "HLL4 + Xxhasher",
        "hll5": "HLL5 + Xxhasher",
        "hll6_WyHash": "HLL6 + WyHash",
        "hll6_wyhash": "HLL6 + WyHash",
        "hll6_xxhash64": "HLL6 + Xxhasher",
        "hll6_XxHash64": "HLL6 + Xxhasher",
        "hll6": "HLL6 + Xxhasher",
        "hll8": "HLL8 + Xxhasher",
        "hll_xxhash64": "HLL6 + Xxhasher",
        "mle": "Our MLE",
        "mle6": "MLE2 + Xxhasher",
        "mle8": "MLE2 + Xxhasher",
        "multiplicities": "Our Multi",
        "tabacpf6": "Tabac's HLL",
        "tabacpf": "Tabac's HLL",
        "tabacplusplus6": "Tabac's HLL++",
        "sa6": "Streaming Algorithms",
        "ce4_wyhash": "Cardinality Estimator",
        "ce4_WyHash": "Cardinality Estimator",
        "ce6_wyhash": "Cardinality Estimator",
        "ce6_WyHash": "Cardinality Estimator",
        "ce6": "Cardinality Estimator",
        "rhll6": "Rust-HLL",
    }[name]


def retrieve_memory_usage(approach_name: str, precision: int) -> float:
    """Returns the memory usage for a given approach and precision."""
    assert precision >= 4
    assert precision <= 16
    path1 = f"tests/statistical_tests_reports/cardinality_{precision}.csv"
    path2 = f"tests/statistical_tests_reports/union_{precision}.csv"
    df = pd.concat(
        [
            pd.read_csv(path1),
            pd.read_csv(path2),
        ]
    )
    # We find the first row that has as first_approach column the approach_name
    # and return the first_memsize column value.
    filtered = df[df["first_approach"] == approach_name]
    if filtered.empty:
        raise ValueError(
            f"Could not find memory usage for {approach_name} and precision {precision}"
        )
    return filtered["first_memsize"].iloc[0]


def parse_name_line(line: str) -> Dict:
    """Parses a name line."""
    # We expect the line to be in the format:
    # hll_cardinality_precision_4_bits_5
    if line.count("_") == 7:
        name, task, _precision, precision, _bits, bits, _hasher, hasher_type = line.split("_")
        assert _precision == "precision"
        assert _bits == "bits"
        assert _hasher == "hasher"
        name = f"{name}{bits}_{hasher_type}"
    elif line.count("_") == 5:
        name, task, _precision, precision, _bits, bits = line.split("_")
        assert _precision == "precision"
        assert _bits == "bits"
        name = f"{name}{bits}"
    else:
        raise NotImplementedError(f"Line has an unexpected format: {line}")
    precision = int(precision)
    bits = int(bits)

    return {
        "name": normalize_name(name),
        "task": task,
        "precision": precision,
        "bits": bits,
        "memory_usage": retrieve_memory_usage(normalize_name(name), precision),
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
        "No change in performance detected.",
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
        except ValueError as exception:
            raise ValueError(f"Error while parsing line: {line}") from exception

        raise NotImplementedError(f"Unknown line: {line}")

    if current_line is not None and current_line:
        rows.append(current_line)

    return pd.DataFrame(rows)


def get_bits_line_style(bits: int) -> Dict:
    """Returns the line style for a given number of bits."""
    return {4: ":", 5: "-.", 6: "-", 8: "--"}[bits]


def get_approach_color(approach_name: str) -> str:
    """Returns the color for a given normalized approach name."""
    colors = pd.read_csv("utilities/colors.csv", index_col="approach")
    return colors.loc[approach_name, "color"]


def get_approach_linestyle(approach_name: str) -> str:
    """Returns the linestyle for a given normalized approach name."""
    linestyles = pd.read_csv("utilities/linestyles.csv", index_col="approach")
    return linestyles.loc[approach_name, "linestyle"]


def get_approach_marker(approach_name: str) -> str:
    """Returns the marker for a given normalized approach name."""
    markers = pd.read_csv("utilities/markers.csv", index_col="approach")
    return markers.loc[approach_name, "marker"]


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
                axes.grid(True, which="both", axis="both")
                axes.fill_between(
                    subdf["memory_usage"],
                    subdf["lower_bound"],
                    subdf["upper_bound"],
                    color=get_approach_color(name),
                    alpha=0.5,
                )
                axes.plot(
                    subdf["memory_usage"],
                    subdf["mean"],
                    marker=get_approach_marker(name),
                    linestyle=get_bits_line_style(bits),
                    color=get_approach_color(name),
                    label=f"{name} - {bits} bits",
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
        try:
            plot_benchmarks(log_path)
        except Exception as e:
            print(f"Error while plotting {log_path}: {e}")
            continue
