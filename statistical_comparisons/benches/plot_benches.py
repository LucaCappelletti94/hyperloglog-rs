"""Python script to load and plot the benchmarks results."""

from typing import List, Dict
import os
from glob import glob
import pandas as pd
import matplotlib.pyplot as plt
from tqdm.auto import tqdm
from sanitize_ml_labels import sanitize_ml_labels


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

def retrieve_feature(
    approach_name: str, precision: int, bits: int, feature: str
) -> float:
    """Returns the memory usage for a given approach and precision."""
    assert precision >= 4
    assert precision <= 18

    if feature == "memory_usage":
        column = "first_memsize"
    elif feature == "mean_error":
        column = "first_mean"
    else:
        raise ValueError(f"Unknown feature: {feature}")

    path1 = f"./statistical_comparisons/statistical_tests_reports/cardinality_{precision}.csv"
    path2 = f"./statistical_comparisons/statistical_tests_reports/union_{precision}.csv"
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
    return filtered[column].iloc[0]


def retrieve_memory_usage(approach_name: str, precision: int, bits: int) -> float:
    """Returns the memory usage for a given approach and precision."""
    return retrieve_feature(approach_name, precision, bits, "memory_usage")


def retrieve_mean_error(approach_name: str, precision: int, bits: int) -> float:
    """Returns the memory usage for a given approach and precision."""
    return retrieve_feature(approach_name, precision, bits, "mean_error")


def parse_name_line(line: str) -> Dict:
    """Parses a name line."""
    # We expect the line to be in the format:
    # hll_cardinality_precision_4_bits_5
    if line.count("_") == 7:
        name, task, _precision, precision, _bits, bits, _hasher, hasher_type = (
            line.split("_")
        )
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
        "memory_usage": retrieve_memory_usage(normalize_name(name), precision, bits),
        "mean_error": retrieve_mean_error(normalize_name(name), precision, bits),
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
        "time_lower_bound": lower_bound,
        "time_mean": mean,
        "time_upper_bound": upper_bound,
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


def plot_benchmarks() -> None:
    """Plots the benchmarks.

    We plot the memory usage on the abscissa and the time on the ordinate,
    with the lower and upper bounds shown both as error bars and as a shaded
    area.
    """

    paths = glob("benches/*.log")

    # We sort the path so that the most recent logs are on top of the list
    # We extract the age of the file from the path, and we use it as a key
    # for the sorting.
    paths.sort(key=lambda path: -os.stat(path).st_mtime)

    cross_task_df = pd.concat([
        load_criterion_log(path)
        for path in paths
    ])

    cross_task_df.to_csv("benches/benchmarks.csv", index=False)

    number_of_tasks = cross_task_df["task"].nunique()
    print(f"Found {len(cross_task_df)} unique benchmarks across {number_of_tasks} tasks")

    cross_task_df["time_x_error"] = cross_task_df["time_mean"] * cross_task_df["mean_error"]

    for task, df in tqdm(
        cross_task_df.groupby("task"),
        total=number_of_tasks,
        desc="Plotting benchmarks",
        leave=False,
    ):
        for y_feature, x_feature in [("time_mean", "memory_usage"), ("time_x_error", "memory_usage")]:
            fig, ax = plt.subplots(
                ncols=2,  # We plot both in linear and log scale, and we also
                nrows=2,  # show the same plots but without the slowest approaches
                figsize=(14, 14),
                dpi=300,
            )

            df = df.sort_values(by=[x_feature])

            # We compute the median of the performance across all approaches

            mean = df[y_feature].mean()

            for name in df["name"].unique():
                for bits in df["bits"].unique():
                    for j, threshold in enumerate([None, mean]):
                        mask = (df["name"] == name) & (df["bits"] == bits)
                        subdf = df[mask]
                        if threshold is not None:
                            if subdf[y_feature].mean() > threshold:
                                continue

                        # If the mask is empty, we skip the plot
                        if subdf.empty:
                            continue
                        for scale, axes in zip(["linear", "log"], ax[j]):
                            axes.set_xscale("log")
                            axes.set_yscale(scale)
                            axes.set_xlabel(sanitize_ml_labels(x_feature))
                            axes.set_ylabel(sanitize_ml_labels(y_feature))
                            if threshold is None:
                                axes.set_title(f"{task} - {scale} scale")
                            else:
                                axes.set_title(
                                    f"{task} - {scale} scale - threshold {threshold:.2e}"
                                )
                            axes.grid(True, which="both", axis="both")
                            # axes.fill_between(
                            #     subdf[x_feature],
                            #     subdf["lower_bound"],
                            #     subdf["upper_bound"],
                            #     color=get_approach_color(name),
                            #     alpha=0.5,
                            # )
                            axes.plot(
                                subdf[x_feature],
                                subdf[y_feature],
                                marker=get_approach_marker(name),
                                markersize=10,
                                linestyle=get_approach_linestyle(name),
                                color=get_approach_color(name),
                                label=f"{name} - {bits} bits",
                            )

            # We position the legend outside the plot, on the right
            # of the second axes. In order to avoid duplicating the
            # legend, we only show it on the second axes.
            ax[0, 0].legend(loc="center left")

            fig.tight_layout()

            figure_path = f"benches/{task}.{y_feature}.{x_feature}.png"

            plt.savefig(figure_path)
            plt.close()


if __name__ == "__main__":
    plot_benchmarks()
