"""Python script to plot the results obtained from the statistical tests."""

import os
from glob import glob
import matplotlib.pyplot as plt
import pandas as pd
from tqdm.auto import tqdm


def plot_cardinality_performance():
    hashset_performance = pd.read_csv("./reports/cardinality-HashSet.csv.gz")

    fig, axes = plt.subplots(1, 1, figsize=(15, 10))

    for other_path in tqdm(
        sorted(glob("./reports/cardinality/*.csv.gz")), leave=False, desc="Plotting"
    ):
        if "wyhash" in other_path.lower():
            continue

        if "B4" in other_path or "B5" in other_path:
            continue

        if "P14" not in other_path:
            continue

        other_performance = pd.read_csv(other_path)

        model_name = other_path.split(os.sep)[-1].split(".")[0]
        model_name = model_name.split("+")[0].strip()

        other_performance["error"] = [
            max(error, 0.00000001)
            for error in other_performance["error"]
        ]

        other_performance["error"] = other_performance["memory_requirement"]# * other_performance["time_requirement"]

        # We sort the predictions and the error by prediction.
        to_plot = list(zip(hashset_performance.prediction, other_performance.error))
        to_plot.sort(key=lambda x: x[0])

        # We smooth the error by taking the mean of the error of the 10 closest points.
        other_performance["error"] = (
            other_performance.error.rolling(50, min_periods=1).mean()
        )

        axes.plot(
            [x[0] for x in to_plot],
            other_performance.error,
            label=model_name,
        )

    axes.set_title("Cardinality Prediction Performance")
    axes.set_xlabel("Cardinality")
    axes.set_xscale("log")
    axes.set_ylabel("Relative Error")
    axes.set_yscale("log")
    axes.legend(ncol=2)

    fig.tight_layout()

    fig.savefig("./reports/cardinality_performance.png")


if __name__ == "__main__":
    plot_cardinality_performance()
