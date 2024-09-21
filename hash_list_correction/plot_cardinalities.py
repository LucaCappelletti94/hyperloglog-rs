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
    name: str
    color: str
    # relative_error_mean: np.ndarray

    @staticmethod
    def from_dictionaries(data: List[Dict[str, float]], name: str, color: str) -> "CardinalitySample":
        """Create a CardinalitySample instance from a dictionary."""
        data = sorted(data, key=lambda sample: sample["exact_cardinality_mean"])

        return CardinalitySample(
            name=name,
            color=color,
            exact_cardinality_mean=np.array(
                [sample["exact_cardinality_mean"] for sample in data]
            ),
            estimated_cardinality_mean=np.array(
                [sample["estimated_cardinality_mean"] for sample in data]
            ),
            absolute_relative_error_mean=np.array(
                [sample["absolute_relative_error_mean"] for sample in data]
            ),
        )

    @property
    def relative_error_mean(self) -> np.ndarray:
        """Return the relative error."""
        exact_cardinality_mean = self.exact_cardinality_mean.copy()
        exact_cardinality_mean[exact_cardinality_mean == 0] = 1

        return (self.exact_cardinality_mean - self.estimated_cardinality_mean) / exact_cardinality_mean

    @property
    def largest_exact_cardinality(self) -> int:
        """Return the largest exact cardinality."""
        if self.exact_cardinality_mean.size == 0:
            return 0

        return self.exact_cardinality_mean.max().astype(int)

    @property
    def largest_estimated_cardinality(self) -> int:
        """Return the largest estimated cardinality."""
        if self.estimated_cardinality_mean.size == 0:
            return 0

        return self.estimated_cardinality_mean.max().astype(int)

    @property
    def largest_absolute_relative_error(self) -> float:
        """Return the largest absolute relative error."""
        if self.absolute_relative_error_mean.size == 0:
            return 0
        return self.absolute_relative_error_mean.max()

    @property
    def smallest_relative_error(self) -> float:
        """Return the smallest relative error."""
        if self.relative_error_mean.size == 0:
            return 0
        return self.relative_error_mean.min()

    @property
    def subtraction(self) -> np.ndarray:
        """Return the subtraction between the exact and estimated cardinalities."""
        return self.exact_cardinality_mean - self.estimated_cardinality_mean

@dataclass
class CardinalitySamplesByModel:
    """Dataclass to store the hash correction data."""

    mean_hash_list_saturation: Optional[float]
    mean_hyperloglog_saturation: Optional[float]
    hyperloglog_fully_imprinted: CardinalitySample
    hyperloglog_imprinted_up: CardinalitySample
    hyperloglog_imprinted_down: CardinalitySample
    hyperloglog: CardinalitySample
    hash_list: CardinalitySample

    @property
    def largest_exact_cardinality(self) -> int:
        """Return the largest exact cardinality."""
        return max(
            self.hyperloglog.largest_exact_cardinality,
            self.hash_list.largest_exact_cardinality,
            self.hyperloglog_fully_imprinted.largest_exact_cardinality,
            self.hyperloglog_imprinted_up.largest_exact_cardinality,
            self.hyperloglog_imprinted_down.largest_exact_cardinality,
        )

    @property
    def largest_estimated_cardinality(self) -> int:
        """Return the largest estimated cardinality."""
        return max(
            self.hyperloglog.largest_estimated_cardinality,
            self.hash_list.largest_estimated_cardinality,
            self.hyperloglog_fully_imprinted.largest_estimated_cardinality,
            self.hyperloglog_imprinted_up.largest_estimated_cardinality,
            self.hyperloglog_imprinted_down.largest_estimated_cardinality,
        )

    @property
    def largest_absolute_relative_error(self) -> float:
        """Return the largest absolute relative error."""
        return max(
            self.hyperloglog.largest_absolute_relative_error,
            self.hash_list.largest_absolute_relative_error,
            self.hyperloglog_fully_imprinted.largest_absolute_relative_error,
            self.hyperloglog_imprinted_up.largest_absolute_relative_error,
            self.hyperloglog_imprinted_down.largest_absolute_relative_error,
        )

    @property
    def smallest_relative_error(self) -> float:
        """Return the smallest relative error."""
        return min(
            self.hyperloglog.smallest_relative_error,
            self.hash_list.smallest_relative_error,
            self.hyperloglog_fully_imprinted.smallest_relative_error,
            self.hyperloglog_imprinted_up.smallest_relative_error,
            self.hyperloglog_imprinted_down.smallest_relative_error,
        )

    def iter_cardinality_samples(self) -> List[CardinalitySample]:
        """Iterate over the cardinality samples."""
        return [
            self.hyperloglog,
            self.hash_list,
            self.hyperloglog_fully_imprinted,
            self.hyperloglog_imprinted_up,
            self.hyperloglog_imprinted_down,
        ]

    @staticmethod
    def from_json(path: str) -> "CardinalitySamplesByModel":
        """Load the hash correction data from the JSON file."""
        data = compress_json.load(path)
        return CardinalitySamplesByModel(
            mean_hash_list_saturation=data.get("mean_hash_list_saturation"),
            mean_hyperloglog_saturation=data.get("mean_hyperloglog_saturation"),
            hyperloglog=CardinalitySample.from_dictionaries(data.get("hyperloglog"), "Uncorrected", "tab:orange"),
            hyperloglog_imprinted_up=CardinalitySample.from_dictionaries(data.get("hyperloglog_imprinted_up"), "Imprinted up", "tab:red"),
            hyperloglog_imprinted_down=CardinalitySample.from_dictionaries(data.get("hyperloglog_imprinted_down"), "Imprinted down", "tab:blue"),
            hyperloglog_fully_imprinted=CardinalitySample.from_dictionaries(data.get("hyperloglog_fully_imprinted"), "Fully imprinted", "tab:green"),
            hash_list=CardinalitySample.from_dictionaries(data.get("hash_list"), "Hash List", "tab:purple"),
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
    fig, axs = plt.subplots(2, 2, figsize=(24, 12))

    for ax in axs.flatten():
        # We plot for the range 1..=5 the number of registers for the current precision.
        if report.largest_exact_cardinality > 5 * 2**correction.precision:
            ax.axvline(
                5 * 2**correction.precision,
                color="tab:brown",
                linestyle="-.",
                label=f"5 * 2^{correction.precision}",
            )
        if report.largest_exact_cardinality > 2**(2**correction.bits) - 1:
            ax.axvline(
                2**(2**correction.bits) - 1,
                color="tab:brown",
                linestyle="--",
                label=f"2^{2**correction.bits} - 1",
            )
    
        if report.mean_hash_list_saturation is not None:
            ax.axvline(
                report.mean_hash_list_saturation,
                linestyle="-.",
                label="Hash List saturation",
                color=report.hash_list.color,
            )
        if report.mean_hyperloglog_saturation is not None:
            ax.axvline(
                report.mean_hyperloglog_saturation,
                linestyle="-.",
                label="HyperLogLog saturation",
                color=report.hyperloglog.color,
            )

    for (i, scale) in enumerate(["linear", "log"]):

        for sample in report.iter_cardinality_samples():
            axs[i][0].plot(
                sample.exact_cardinality_mean,
                sample.absolute_relative_error_mean,
                alpha=0.7,
                label=f"{sample.name} absolute relative errors",
                color=sample.color,
            )

            axs[i][0].plot(
                sample.exact_cardinality_mean,
                sample.relative_error_mean,
                label=f"{sample.name} absolute relative errors",
                linestyle="--",
                alpha=0.7,
                color=sample.color,
            )

            axs[i][1].plot(
                sample.exact_cardinality_mean,
                sample.subtraction,
                label=f"{sample.name} subtraction",
                linestyle="-",
                alpha=0.7,
                color=sample.color,
            )

        # We plot as dots the cardinalities that appear in the hash correction data.

        if correction.hyperloglog_cardinalities.size > 0:
            relative_errors = []
            subtractions = []
            cardinalities_covered = set()

            for exact, estimated in zip(
                report.hyperloglog_fully_imprinted.exact_cardinality_mean,
                report.hyperloglog_fully_imprinted.estimated_cardinality_mean,
            ):
                if (
                    round(estimated) in correction.hyperloglog_cardinalities
                    and round(estimated) not in cardinalities_covered
                ):
                    relative_errors.append((exact, (exact - estimated) / max(exact, 1)))
                    subtractions.append((exact, exact - estimated))
                    cardinalities_covered.add(round(estimated))

            axs[i][0].scatter(
                *zip(*relative_errors),
                marker=".",
                label="HyperLogLog correction",
                color=report.hyperloglog_fully_imprinted.color,
            )
            axs[i][1].scatter(
                *zip(*subtractions),
                marker=".",
                label="HyperLogLog correction",
                color=report.hyperloglog_fully_imprinted.color,
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

        axs[i][0].scatter(
            *zip(*relative_errors),
            marker=".",
            label="HashList correction",
            color=report.hash_list.color,
        )
        axs[i][1].scatter(
            *zip(*subtractions),
            marker=".",
            label="HashList correction",
            color=report.hash_list.color,
        )

        expected_error = expected_hll_error(correction.precision)

        steps = np.array([
            report.largest_exact_cardinality * step / 1000
            for step in range(0, 1000)
        ])

        if 1 not in steps:
            steps = np.append(steps, 1)

        steps.sort()   

        if report.largest_absolute_relative_error >= expected_hll_error(correction.precision):
            # We plot an horizontal line with the expected HLL error.
            axs[i][0].axhline(
                expected_hll_error(correction.precision),
                color="tab:red",
                linestyle="-.",
                label="Expected HLL error",
            )
            axs[i][1].plot(
                steps,
                steps * expected_error,
                color="tab:red",
                linestyle="-.",
                label="Expected HLL error",
            )

        if report.smallest_relative_error <= -expected_hll_error(correction.precision):
            axs[i][0].axhline(
                -expected_hll_error(correction.precision),
                color="tab:red",
                linestyle="-.",
            )         

            axs[i][1].plot(
                steps,
                -steps * expected_error,
                linestyle="-.",
                color="tab:red",
            )

        axs[i][0].set_xlabel(f"Exact cardinality ({scale})")
        axs[i][1].set_xlabel(f"Exact cardinality ({scale})")
        axs[i][0].set_ylabel("Relative error")
        axs[i][0].legend(ncol=2, loc="lower right")
        axs[i][1].legend(ncol=2, loc="lower right")
        axs[i][0].grid(which="both", linestyle="--", alpha=0.5)
        axs[i][1].grid(which="both", linestyle="--", alpha=0.5)
        axs[i][1].set_ylabel("Exact - Estimated")
        axs[i][1].set_yscale("symlog")
        axs[i][0].set_xscale(scale)
        axs[i][1].set_xscale(scale)
        axs[i][0].set_title(f"Relative errors p={correction.precision}, b={correction.bits}")
        axs[i][1].set_title(f"Subtraction p={correction.precision}, b={correction.bits}")

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
