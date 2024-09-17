"""Script to plot CSVs with the exact and estimated cardinalities."""

from typing import List
from glob import glob
from multiprocessing import Pool
import os
from dataclasses import dataclass, field
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
from tqdm import tqdm
import compress_json


@dataclass
class HashCorrection:
    """Dataclass to store the hash correction data."""

    precision: int
    bits: int
    hashlist_largest_maximal_cardinality: int
    hashlist_mean_maximal_cardinality: int
    hashlist_smallest_maximal_cardinality: int
    hashlist_relative_errors: List[int]
    hyperloglog_relative_errors: List[int]
    hashlist_cardinalities: List[int]
    hyperloglog_cardinalities: List[int]
    hyperloglog_slope: float

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
            hyperloglog_slope=data.get("hyperloglog_slope"),
        )


def expected_hll_error(precision: int) -> float:
    """Compute the expected error of the HyperLogLog algorithm."""
    return 1.04 / (precision**0.5)


def plot_cardinality(path_json: str):
    """Load the reports and plot the histograms, boxplots and relative error plots."""
    hashlist_path = path_json.replace(".hashlist.json", ".hashlist.csv.gz")
    hyperloglog_path = path_json.replace(".hashlist.json", ".hyperloglog.csv.gz")
    image_path = path_json.replace(".hashlist.json", ".png")
    hash_correction = HashCorrection.from_json(path_json)
    fig, axs = plt.subplots(2, 1, figsize=(12, 12), sharex=False, sharey=False)
    has_negative_deltas = False

    largest_exact_cardinality = 0
    largest_estimated_cardinality = 0

    for path, name, correction in [
        (hashlist_path, "Hashlist", hash_correction.hashlist_cardinalities),
        (
            hyperloglog_path,
            "HyperLogLog",
            hash_correction.hyperloglog_cardinalities,
        ),
    ]:
        if len(correction) == 0:
            continue

        try:
            report = pd.read_csv(path)
        except FileNotFoundError:
            return
        largest_exact_cardinality = int(max(largest_exact_cardinality, report.y.max()))
        largest_estimated_cardinality = max(
            largest_estimated_cardinality, report.x.max()
        )
        report.sort_values(by="y", inplace=True)

        axs[0].plot(
            report.y,
            [
                abs(estimated - exact) / max(exact, 1)
                for exact, estimated in zip(report.y, report.x)
            ],
            label=f"{name} relative error",
            linestyle="-",
            alpha=0.7,
        )

        axs[1].plot(
            report.y,
            [exact - estimated for exact, estimated in zip(report.y, report.x)],
            label=f"{name} subtraction",
            linestyle="-",
            alpha=0.7,
        )

        if any(exact - estimated < 0 for exact, estimated in zip(report.y, report.x)):
            has_negative_deltas = True

        # We plot as dots the cardinalities that appear in the hash correction data.

        in_correction = []
        cardinalities_covered = set()

        for exact, estimated in zip(report.y, report.x):
            if (
                round(estimated) in correction
                and round(estimated) not in cardinalities_covered
            ):
                in_correction.append((exact, abs(estimated - exact) / max(exact, 1)))
                cardinalities_covered.add(round(estimated))

        axs[0].scatter(
            *zip(*in_correction),
            label=f"{name} correction",
        )

        in_correction = []
        cardinalities_covered = set()

        for exact, estimated in zip(report.y, report.x):
            if (
                round(estimated) in correction
                and round(estimated) not in cardinalities_covered
            ):
                in_correction.append((exact, exact - estimated))
                cardinalities_covered.add(round(estimated))

        axs[1].scatter(
            *zip(*in_correction),
            label=f"{name} correction subtraction",
        )

    if len(hash_correction.hyperloglog_cardinalities) > 0:
        # We plot the continuation of the expected estimation relative error and subtraction
        # according to the Slope and Intercept we have determined for this particular hash correction.
        extended_estimated_cardinalities = np.arange(
            largest_estimated_cardinality + 1, largest_estimated_cardinality * 3 / 2
        )
        extended_exact_cardinalities = np.arange(
            largest_exact_cardinality + 1, largest_exact_cardinality + len(extended_estimated_cardinalities)
        )

        largest_exact_cardinality = max(largest_exact_cardinality, extended_exact_cardinalities[-1])

        extended_predicted_cardinalities = (
            hash_correction.hyperloglog_slope * (extended_estimated_cardinalities - largest_estimated_cardinality) + largest_estimated_cardinality
        )

        axs[0].plot(
            extended_exact_cardinalities,
            [
                abs(estimated - exact) / max(exact, 1)
                for exact, estimated in zip(
                    extended_exact_cardinalities, extended_predicted_cardinalities
                )
            ],
            label="LS prediction relative error",
            linestyle="-",
            color="tab:pink",
            alpha=0.9,
        )

        axs[1].plot(
            extended_exact_cardinalities,
            [
                exact - estimated
                for exact, estimated in zip(
                    extended_exact_cardinalities, extended_predicted_cardinalities
                )
            ],
            label="LS prediction subtraction",
            linestyle="-",
            color="tab:pink",
            alpha=0.9,
        )

    # We plot an horizontal line with the expected HLL error.
    axs[0].axhline(
        expected_hll_error(hash_correction.precision),
        color="tab:red",
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
            range(0, largest_exact_cardinality + 1),
            [
                -expected_error * max(exact, 1)
                for exact in range(0, largest_exact_cardinality + 1)
            ],
            [
                expected_error * max(exact, 1)
                for exact in range(0, largest_exact_cardinality + 1)
            ],
            color="tab:red",
            alpha=0.3,
            label="Expected HLL error",
        )
    else:
        axs[1].fill_between(
            range(0, largest_exact_cardinality + 1),
            [
                0
                for exact in range(0, largest_exact_cardinality + 1)
            ],
            [
                expected_error * max(exact, 1)
                for exact in range(0, largest_exact_cardinality + 1)
            ],
            color="tab:red",
            alpha=0.3,
            label="Expected HLL error",
        )

    for ax in axs:
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
            color="tab:green",
            linestyle="--",
            label="Smallest hashlist cardinality",
        )

        # We plot a vertical line at the point of largest hashlist cardinality.
        ax.axvline(
            hash_correction.hashlist_largest_maximal_cardinality,
            color="tab:blue",
            linestyle="--",
            label="Largest hashlist cardinality",
        )

        # We plot for the range 1..=5 the number of registers for the current precision.
        precision = hash_correction.precision

        ax.axvline(
            5 * 2**precision,
            color="tab:purple",
            linestyle="--",
            label=f"5 * 2^{precision}",
        )

    axs[0].set_xlabel("Exact cardinality")
    axs[0].set_ylabel("Relative error")
    axs[0].legend(ncol=2, loc="upper right")
    axs[0].grid(which="both", linestyle="--", alpha=0.5)
    axs[1].set_xlabel("Exact cardinality")
    axs[1].set_ylabel("Exact - Estimated")
    axs[1].set_yscale("symlog")
    # axs[0].set_xscale("log")
    # axs[1].set_xscale("log")
    axs[1].legend(ncol=2, loc="lower right")
    axs[1].grid(which="both", linestyle="--", alpha=0.5)
    axs[0].set_title(
        f"Relative errors p={hash_correction.precision}, b={hash_correction.bits}"
    )
    axs[1].set_title(
        f"Subtraction p={hash_correction.precision}, b={hash_correction.bits}"
    )

    fig.tight_layout()

    plt.savefig(image_path)
    plt.close()


def plot_cardinalities():
    """Load the reports and plot the histograms, boxplots and relative error plots."""
    paths = glob("*.hashlist.json")
    with Pool() as pool:
        list(
            tqdm(
                pool.imap_unordered(plot_cardinality, paths),
                total=len(paths),
                leave=False,
            )
        )


if __name__ == "__main__":
    plot_cardinalities()
