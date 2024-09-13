"""Script to plot CSVs with the exact and estimated cardinalities."""

from typing import List
from glob import glob
from dataclasses import dataclass, field
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
    hashlist_largest_maximal_cardinality: int
    hashlist_mean_maximal_cardinality: int
    hashlist_smallest_maximal_cardinality: int
    hashlist_relative_errors: List[int] = field(default_factory=list)
    hyperloglog_relative_errors: List[int] = field(default_factory=list)
    hashlist_cardinalities: List[int] = field(default_factory=list)
    hyperloglog_cardinalities: List[int] = field(default_factory=list)

    @staticmethod
    def from_json(path: str) -> "HashCorrection":
        """Load the hash correction data from the JSON file."""
        data = compress_json.load(path)
        return HashCorrection(
            precision=data.get("precision"),
            bits=data.get("bits"),
            hashlist_largest_maximal_cardinality=data.get(
                "hashlist_largest_maximal_cardinality"
            ),
            hashlist_mean_maximal_cardinality=data.get(
                "hashlist_mean_maximal_cardinality"
            ),
            hashlist_smallest_maximal_cardinality=data.get(
                "hashlist_smallest_maximal_cardinality"
            ),
            hashlist_relative_errors=data.get("hashlist_relative_errors"),
            hyperloglog_relative_errors=data.get("hyperloglog_relative_errors"),
            hashlist_cardinalities=data.get("hashlist_cardinalities"),
            hyperloglog_cardinalities=data.get("hyperloglog_cardinalities"),
        )


def expected_hll_error(precision: int) -> float:
    """Compute the expected error of the HyperLogLog algorithm."""
    return 1.04 / (precision**0.5)


def plot_cardinalities():
    """Load the reports and plot the histograms, boxplots and relative error plots."""
    for path_json in tqdm(glob("*_*.hashlist.json")):

        hashlist_path = path_json.replace(".hashlist.json", ".hashlist.csv.gz")
        hyperloglog_path = path_json.replace(".hashlist.json", ".hyperloglog.csv.gz")
        image_path = path_json.replace(".hashlist.json", ".png")
        hash_correction = HashCorrection.from_json(path_json)
        fig, axs = plt.subplots(2, 1, figsize=(12, 12), sharex=False, sharey=False)
        has_negative_deltas = False

        for path, name, correction in [
            (hashlist_path, "Hashlist", hash_correction.hashlist_cardinalities),
            (hyperloglog_path, "HyperLogLog", hash_correction.hyperloglog_cardinalities),
        ]:
            report = pd.read_csv(path)
            report.sort_values(by="exact_cardinality", inplace=True)

            axs[0].plot(
                report.exact_cardinality,
                [
                    abs(estimated - exact) / max(exact, 1)
                    for exact, estimated in zip(
                        report.exact_cardinality, report.cardinality
                    )
                ],
                label=f"{name} relative error",
                linestyle="-",
                marker="x" if hash_correction.precision <= 5 else None,
                alpha=0.7,
            )

            axs[1].plot(
                report.exact_cardinality,
                [
                    exact - estimated 
                    for exact, estimated in zip(
                        report.exact_cardinality, report.cardinality
                    )
                ],
                label=f"{name} subtraction",
                linestyle="-",
                marker="x" if hash_correction.precision <= 5 else None,
                alpha=0.7,
            )

            if any(exact - estimated < 0 for exact, estimated in zip(
                        report.exact_cardinality, report.cardinality)):
                has_negative_deltas = True

            # We plot as dots the cardinalities that appear in the hash correction data.

            in_correction = [
                (exact, abs(estimated - exact) / max(exact, 1))
                for exact, estimated in zip(
                    report.exact_cardinality, report.cardinality
                )
                if round(estimated) in correction
            ]

            axs[0].scatter(
                *zip(*in_correction),
                label=f"{name} correction",
            )

            in_correction = [
                (exact, exact - estimated)
                for exact, estimated in zip(
                    report.exact_cardinality, report.cardinality
                )
                if round(estimated) in correction
            ]

            axs[1].scatter(
                *zip(*in_correction),
                label=f"{name} correction subtraction",
            )

        # We plot an horizontal line with the expected HLL error.
        axs[0].axhline(
            expected_hll_error(hash_correction.precision),
            color="tab:green",
            linestyle="--",
            label="Expected HLL error",
        )

        # We analogously plot the expected HLL error for the subtraction,
        # which is not relative but absolute, and thus depends on the cardinality.
        # As such, we need to consider the plus and minus expected error, which we 
        # best display as a shaded area. If there are no negative values in this particular
        # plot, we only display the positive expected error.

        expected_error = expected_hll_error(hash_correction.precision)
        if has_negative_deltas:
            axs[1].fill_between(
                np.arange(0, report.exact_cardinality.max() + 1),
                -expected_error * np.arange(0, report.exact_cardinality.max() + 1),
                expected_error * np.arange(0, report.exact_cardinality.max() + 1),
                color="tab:green",
                alpha=0.3,
                label="Expected HLL error",
            )
        else:
            axs[1].fill_between(
                np.arange(0, report.exact_cardinality.max() + 1),
                0,
                expected_error * np.arange(0, report.exact_cardinality.max() + 1),
                color="tab:green",
                alpha=0.3,
                label="Expected HLL error",
            )

        for ax in axs:
            # We plot a vertical line at the point of maximal hashlist cardinality.
            ax.axvline(
                hash_correction.hashlist_largest_maximal_cardinality,
                color="tab:blue",
                linestyle="--",
                label="Largest hashlist cardinality",
            )

            # We plot a vertical line at the point of mean hashlist cardinality.
            ax.axvline(
                hash_correction.hashlist_mean_maximal_cardinality,
                color="tab:orange",
                linestyle="--",
                label="Mean hashlist cardinality",
            )

            # We plot a vertical line at the point of smallest hashlist cardinality.
            ax.axvline(
                hash_correction.hashlist_smallest_maximal_cardinality,
                color="tab:red",
                linestyle="--",
                label="Smallest hashlist cardinality",
            )

            # We plot for the range 1..=5 the number of registers for the current precision.
            precision = hash_correction.precision

            for i in range(1, 6):
                ax.axvline(
                    i * 2**precision,
                    color="tab:purple",
                    linestyle="--",
                    label=f"bits=2^{precision} * {i}",
                )

        axs[0].set_xlabel("Exact cardinality")
        axs[0].set_ylabel("Relative error")
        axs[0].legend()
        axs[0].grid(which="both", linestyle="--", alpha=0.5)
        axs[1].set_xlabel("Exact cardinality")
        axs[1].set_ylabel("Subtraction")
        axs[1].legend()
        axs[1].grid(which="both", linestyle="--", alpha=0.5)

        fig.tight_layout()

        plt.savefig(image_path)
        plt.close()


if __name__ == "__main__":
    plot_cardinalities()
