"""Script to plot CSVs with the exact and estimated cardinalities."""

from typing import List, Optional, Dict
from glob import glob
from multiprocessing import Pool
from dataclasses import dataclass
import matplotlib.pyplot as plt
from tqdm import tqdm
import numpy as np
import compress_json


@dataclass
class Correction:
    """Dataclass to store the hash correction data."""

    precision: int
    bits: int
    hash_list_bias: np.ndarray
    hyperloglog_relative_bias: np.ndarray
    hash_list_cardinalities: np.ndarray
    hyperloglog_cardinalities: np.ndarray

    @staticmethod
    def from_json(path: str) -> "Correction":
        """Load the hash correction data from the JSON file."""
        data = compress_json.load(path)
        return Correction(
            precision=data.get("precision"),
            bits=data.get("bits"),
            hash_list_bias=np.array(data.get("hash_list_bias")),
            hyperloglog_relative_bias=np.array(data.get("hyperloglog_relative_bias")),
            hash_list_cardinalities=np.array(data.get("hash_list_cardinalities")),
            hyperloglog_cardinalities=np.array(data.get("hyperloglog_cardinalities")),
        )


@dataclass
class CardinalitySample:
    """Dataclass to store cardinality samples."""

    exact_cardinality_mean: np.ndarray
    estimated_cardinality_mean: np.ndarray
    absolute_relative_error_mean: np.ndarray
    relative_error_mean: np.ndarray

    @staticmethod
    def from_dictionaries(data: List[Dict[str, float]]) -> "CardinalitySample":
        """Create a CardinalitySample instance from a dictionary."""
        return CardinalitySample(
            exact_cardinality_mean=np.array(
                [sample["exact_cardinality_mean"] for sample in data]
            ),
            estimated_cardinality_mean=np.array(
                [sample["estimated_cardinality_mean"] for sample in data]
            ),
            absolute_relative_error_mean=np.array(
                [sample["absolute_relative_error_mean"] for sample in data]
            ),
            relative_error_mean=np.array(
                [sample["relative_error_mean"] for sample in data]
            ),
        )

    @property
    def largest_exact_cardinality(self) -> int:
        """Return the largest exact cardinality."""
        return self.exact_cardinality_mean.max().astype(int)

    @property
    def largest_estimated_cardinality(self) -> int:
        """Return the largest estimated cardinality."""
        return self.estimated_cardinality_mean.max().astype(int)

    @property
    def subtraction(self) -> np.ndarray:
        """Return the subtraction between the exact and estimated cardinalities."""
        return self.exact_cardinality_mean - self.estimated_cardinality_mean

    def has_negative_deltas(self) -> bool:
        """Return whether the subtraction has negative values."""
        return (self.exact_cardinality_mean - self.estimated_cardinality_mean < 0).any()

@dataclass
class CardinalitySamplesByModel:
    """Dataclass to store the hash correction data."""

    mean_hash_list_saturation: Optional[float]
    mean_hyperloglog_saturation: Optional[float]
    hyperloglog: CardinalitySample
    hash_list: CardinalitySample

    @property
    def largest_exact_cardinality(self) -> int:
        """Return the largest exact cardinality."""
        return max(
            self.hyperloglog.largest_exact_cardinality,
            self.hash_list.largest_exact_cardinality,
        )

    @property
    def largest_estimated_cardinality(self) -> int:
        """Return the largest estimated cardinality."""
        return max(
            self.hyperloglog.largest_estimated_cardinality,
            self.hash_list.largest_estimated_cardinality,
        )
    
    def has_negative_deltas(self) -> bool:
        """Return whether the subtraction has negative values."""
        return self.hyperloglog.has_negative_deltas() or self.hash_list.has_negative_deltas()

    @staticmethod
    def from_json(path: str) -> "CardinalitySamplesByModel":
        """Load the hash correction data from the JSON file."""
        data = compress_json.load(path)
        return CardinalitySamplesByModel(
            mean_hash_list_saturation=data.get("mean_hash_list_saturation"),
            mean_hyperloglog_saturation=data.get("mean_hyperloglog_saturation"),
            hyperloglog=CardinalitySample.from_dictionaries(data.get("hyperloglog")),
            hash_list=CardinalitySample.from_dictionaries(data.get("hash_list")),
        )


def expected_hll_error(precision: int) -> float:
    """Compute the expected error of the HyperLogLog algorithm."""
    return 1.04 / (precision**0.5)


def plot_cardinality(path_json: str):
    """Load the reports and plot the histograms, boxplots and relative error plots."""
    report_path = path_json.replace(".correction.json", ".report.json")
    image_path = path_json.replace(".correction.json", ".png")
    correction = Correction.from_json(path_json)
    report = CardinalitySamplesByModel.from_json(report_path)
    fig, axs = plt.subplots(2, 1, figsize=(12, 12), sharex=False, sharey=False)

    hyperloglog_color = "tab:orange"
    hash_list_color = "tab:blue"

    axs[0].plot(
        report.hyperloglog.exact_cardinality_mean,
        report.hyperloglog.absolute_relative_error_mean,
        label="HyperLogLog absolute relative errors",
        linestyle="-",
        alpha=0.7,
        color=hyperloglog_color,
    )

    axs[0].plot(
        report.hyperloglog.exact_cardinality_mean,
        report.hyperloglog.relative_error_mean,
        label="HyperLogLog relative errors",
        linestyle="--",
        alpha=0.7,
        color=hyperloglog_color,
    )

    axs[0].plot(
        report.hash_list.exact_cardinality_mean,
        report.hash_list.absolute_relative_error_mean,
        label="Hash List absolute relative errors",
        linestyle="-",
        alpha=0.7,
        color=hash_list_color,
    )

    axs[0].plot(
        report.hash_list.exact_cardinality_mean,
        report.hash_list.relative_error_mean,
        label="Hash List relative errors",
        linestyle="--",
        alpha=0.7,
        color=hash_list_color,
    )

    axs[1].plot(
        report.hyperloglog.exact_cardinality_mean,
        report.hyperloglog.subtraction,
        label="HyperLogLog subtraction",
        linestyle="-",
        alpha=0.7,
        color=hyperloglog_color,
    )

    axs[1].plot(
        report.hash_list.exact_cardinality_mean,
        report.hash_list.subtraction,
        label="Hash List subtraction",
        linestyle="-",
        alpha=0.7,
        color=hash_list_color,
    )

    for ax in axs:
        if report.mean_hash_list_saturation is not None:
            ax.axvline(
                report.mean_hash_list_saturation,
                linestyle="-.",
                label="Hash List saturation",
                color=hash_list_color,
            )
        if report.mean_hyperloglog_saturation is not None:
            ax.axvline(
                report.mean_hyperloglog_saturation,
                linestyle="-.",
                label="HyperLogLog saturation",
                color=hyperloglog_color,
            )

    # We plot as dots the cardinalities that appear in the hash correction data.

    relative_errors = []
    subtractions = []
    cardinalities_covered = set()

    for exact, estimated in zip(
        report.hyperloglog.exact_cardinality_mean,
        report.hyperloglog.estimated_cardinality_mean,
    ):
        if (
            round(estimated) in correction.hyperloglog_cardinalities
            and round(estimated) not in cardinalities_covered
        ):
            relative_errors.append((exact, (exact - estimated) / max(exact, 1)))
            subtractions.append((exact, exact - estimated))
            cardinalities_covered.add(round(estimated))

    axs[0].scatter(
        *zip(*relative_errors),
        marker=".",
        label="HyperLogLog correction",
        color=hyperloglog_color,
    )
    axs[1].scatter(
        *zip(*subtractions),
        marker=".",
        label="HyperLogLog correction",
        color=hyperloglog_color,
    )

    relative_errors = []
    subtractions = []
    cardinalities_covered = set()

    for exact, estimated in zip(
        report.hash_list.exact_cardinality_mean, report.hash_list.estimated_cardinality_mean
    ):
        if (
            round(estimated) in correction.hash_list_cardinalities
            and round(estimated) not in cardinalities_covered
        ):
            relative_errors.append((exact, (exact - estimated) / max(exact, 1)))
            subtractions.append((exact, exact - estimated))
            cardinalities_covered.add(round(estimated))

    axs[0].scatter(
        *zip(*relative_errors),
        marker=".",
        label="HashList correction",
        color=hash_list_color,
    )
    axs[1].scatter(
        *zip(*subtractions),
        marker=".",
        label="HashList correction",
        color=hash_list_color,
    )

    # We plot an horizontal line with the expected HLL error.
    axs[0].axhline(
        expected_hll_error(correction.precision),
        color="tab:red",
        linestyle="-.",
        label="Expected HLL error",
    )

    axs[0].axhline(
        -expected_hll_error(correction.precision),
        color="tab:red",
        linestyle="-.",
    )

    # We analogously plot the expected HLL error for the subtraction,
    # which is not relative but absolute, and thus depends on the cardinality.
    # As such, we need to consider the plus and minus expected error, which we
    # best display as a shaded area. If there are no negative values in this particular
    # plot, we only display the positive expected error.

    expected_error = expected_hll_error(correction.precision)

    axs[1].plot(
        [0, 1, report.largest_exact_cardinality],
        [0, expected_error, expected_error * report.largest_exact_cardinality],
        color="tab:red",
        linestyle="-.",
        label="Expected HLL error",
    )
    if report.has_negative_deltas():
        axs[1].plot(
            [0, 1, report.largest_exact_cardinality],
            [0, -expected_error, -expected_error * report.largest_exact_cardinality],
            linestyle="-.",
            color="tab:red",
        )

    for ax in axs:
        # We plot for the range 1..=5 the number of registers for the current precision.
        ax.axvline(
            5 * 2**correction.precision,
            color="tab:purple",
            linestyle="-.",
            label=f"5 * 2^{correction.precision}",
        )
        if report.largest_exact_cardinality > 2**(2**correction.bits) - 1:
            ax.axvline(
                2**(2**correction.bits) - 1,
                color="tab:purple",
                linestyle="--",
                label=f"2^{2**correction.bits} - 1",
            )

    axs[0].set_xlabel("Exact cardinality")
    axs[0].set_ylabel("Relative error")
    axs[0].legend(ncol=2, loc="lower right")
    axs[1].legend(ncol=2, loc="lower right")
    axs[0].grid(which="both", linestyle="--", alpha=0.5)
    axs[1].grid(which="both", linestyle="--", alpha=0.5)
    axs[1].set_xlabel("Exact cardinality")
    axs[1].set_ylabel("Exact - Estimated")
    axs[1].set_yscale("symlog")
    axs[0].set_xscale("log")
    axs[1].set_xscale("log")
    axs[0].set_title(f"Relative errors p={correction.precision}, b={correction.bits}")
    axs[1].set_title(f"Subtraction p={correction.precision}, b={correction.bits}")

    fig.tight_layout()

    plt.savefig(image_path)
    plt.close()


def plot_cardinalities():
    """Load the reports and plot the histograms, boxplots and relative error plots."""
    paths = glob("*.correction.json")
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
