"""Script to plot CSVs with the exact and estimated cardinalities."""

from typing import List
from glob import glob
from dataclasses import dataclass, field
import math
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
from tqdm.auto import tqdm
import compress_json


@dataclass
class HashCorrection:
    """Dataclass to store the hash correction data."""

    precision: int
    bits: int
    relative_errors: List[float] = field(default_factory=list)
    cardinalities: List[int] = field(default_factory=list)

    @staticmethod
    def from_json(path: str) -> "HashCorrection":
        """Load the hash correction data from the JSON file."""
        data = compress_json.load(path)
        return HashCorrection(
            precision=data.get("precision"),
            bits=data.get("bits"),
            relative_errors=data.get("relative_errors", []),
            cardinalities=data.get("cardinalities", []),
        )


def plot_cardinalities():
    """Load the reports and plot the histograms, boxplots and relative error plots."""
    for path in tqdm(glob("*hash<*.csv")):
        report = pd.read_csv(path)
        report.sort_values(by="exact_cardinality", inplace=True)

        hash_correction = HashCorrection.from_json(path.replace(".csv", ".json"))

        _fig, axs = plt.subplots(1, 1, figsize=(6, 6), sharex=False, sharey=False)
        axs.plot(
            report.exact_cardinality,
            [
                (exact - estimated) / max(exact, 1)
                for exact, estimated in zip(report.exact_cardinality, report.cardinality)
            ],
            label="Exact - Estimated",
            linestyle="--",
            alpha=0.7,
        )

        # We plot as red dots the cardinalities that appear in the hash correction data.

        in_hash_correction = [
            (exact, (exact - estimated) / max(exact, 1))
            for exact, estimated in zip(report.exact_cardinality, report.cardinality)
            if round(estimated) in hash_correction.cardinalities
        ]

        axs.scatter(
            *zip(*in_hash_correction),
            color="red",
            label="In hash correction",
        )

        axs.set_xlabel("Exact cardinality")
        axs.set_ylabel("Exact - Estimated")
        axs.legend()

        path = path.replace(".csv", "")

        plt.savefig(f"{path}.png")
        plt.close()


if __name__ == "__main__":
    plot_cardinalities()
