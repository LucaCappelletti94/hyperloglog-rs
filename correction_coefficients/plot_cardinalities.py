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

    @staticmethod
    def from_dictionaries(
        data: List[Dict[str, float]], name: str, color: str
    ) -> "CardinalitySample":
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

        return (
            self.exact_cardinality_mean - self.estimated_cardinality_mean
        ) / exact_cardinality_mean

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
    reports: List[CardinalitySample]

    @property
    def correction_hash_list(self) -> CardinalitySample:
        """Return the hash list correction."""
        for report in self.reports:
            if "HashList" in report.name:
                return report

        raise ValueError("Hash List not found")

    @property
    def correction_hyperloglog(self) -> CardinalitySample:
        """Return the HyperLogLog correction."""
        for report in self.reports:
            if "HyperLogLog" in report.name:
                return report

        raise ValueError("HyperLogLog not found")

    @property
    def largest_exact_cardinality(self) -> int:
        """Return the largest exact cardinality."""
        return max(report.largest_exact_cardinality for report in self.reports)

    @property
    def largest_estimated_cardinality(self) -> int:
        """Return the largest estimated cardinality."""
        return max(report.largest_estimated_cardinality for report in self.reports)

    @property
    def largest_absolute_relative_error(self) -> float:
        """Return the largest absolute relative error."""
        return max(report.largest_absolute_relative_error for report in self.reports)

    @property
    def smallest_relative_error(self) -> float:
        """Return the smallest relative error."""
        return min(report.smallest_relative_error for report in self.reports)

    def iter_cardinality_samples(self) -> List[CardinalitySample]:
        """Iterate over the cardinality samples."""
        return self.reports

    @property
    def hash_list_color(self) -> str:
        """Return the color of the hash list."""
        for report in self.reports:
            if "HashList" in report.name:
                return report.color

        raise ValueError("Hash List not found")

    @property
    def hyperloglog_color(self) -> str:
        """Return the color of the HyperLogLog."""
        for report in self.reports:
            if "HyperLogLog" in report.name:
                return report.color

        raise ValueError("HyperLogLog not found")

    @staticmethod
    def from_json(path: str) -> "CardinalitySamplesByModel":
        """Load the hash correction data from the JSON file."""
        data = compress_json.load(path)
        return CardinalitySamplesByModel(
            mean_hash_list_saturation=data.get("mean_hash_list_saturation"),
            mean_hyperloglog_saturation=data.get("mean_hyperloglog_saturation"),
            reports=[
                CardinalitySample.from_dictionaries(
                    data.get("hyperloglog"), "HyperLogLog", "tab:orange"
                ),
                CardinalitySample.from_dictionaries(
                    data.get("hash_list"), "HashList", "tab:purple"
                ),
            ],
        )


def expected_hll_error(precision: int) -> float:
    """Compute the expected error of the HyperLogLog algorithm."""
    return 1.04 / (precision**0.5)


def plot_cardinality(path_json: str):
    """Load the reports and plot the histograms, boxplots and relative error plots."""
    report_path = path_json.replace(".correction.json", ".report.json")
    image_path = path_json.replace(".correction.json", ".png")

    try:
        correction = Correction.from_json(path_json)
        report = CardinalitySamplesByModel.from_json(report_path)
    except FileNotFoundError:
        print(f"File not found: {path_json}")
        return
    fig, axs = plt.subplots(2, 2, figsize=(18, 12))

    for ax in axs.flatten():
        # We plot for the range 1..=5 the number of registers for the current precision.
        if report.largest_exact_cardinality > 5 * 2**correction.precision:
            ax.axvline(
                5 * 2**correction.precision,
                color="tab:brown",
                linestyle="-.",
                label=f"5 * 2^{correction.precision}",
            )
        if report.largest_exact_cardinality > 7.5 * 2**correction.precision:
            ax.axvline(
                7.5 * 2**correction.precision,
                color="tab:green",
                linestyle=":",
                label=f"7.5 * 2^{correction.precision}",
            )
        if report.largest_exact_cardinality > 2 ** (2**correction.bits) - 1:
            ax.axvline(
                2 ** (2**correction.bits) - 1,
                color="tab:brown",
                linestyle="--",
                label=f"2^{2**correction.bits} - 1",
            )

        if report.mean_hash_list_saturation is not None:
            ax.axvline(
                report.mean_hash_list_saturation,
                linestyle="-.",
                label="Hash List saturation",
                color=report.hash_list_color,
            )
        if report.mean_hyperloglog_saturation is not None:
            ax.axvline(
                report.mean_hyperloglog_saturation,
                linestyle="-.",
                label="HyperLogLog saturation",
                color=report.hyperloglog_color,
            )

    for i, scale in enumerate(["linear", "symlog"]):

        for sample in report.iter_cardinality_samples():
            axs[i][0].plot(
                sample.exact_cardinality_mean,
                sample.absolute_relative_error_mean,
                label=f"{sample.name} absolute relative errors",
                color=sample.color,
            )

            axs[i][0].plot(
                sample.exact_cardinality_mean,
                sample.relative_error_mean,
                label=f"{sample.name} absolute relative errors",
                linestyle="--",
                color=sample.color,
            )

            axs[i][1].plot(
                sample.exact_cardinality_mean,
                sample.subtraction,
                label=f"{sample.name} subtraction",
                linestyle="-",
                color=sample.color,
            )

        # We plot as dots the cardinalities that appear in the hash correction data.

        relative_errors = []
        subtractions = []
        cardinalities_covered = set()

        for exact, estimated in zip(
            report.correction_hyperloglog.exact_cardinality_mean,
            report.correction_hyperloglog.estimated_cardinality_mean,
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
            color=report.correction_hyperloglog.color,
        )
        axs[i][1].scatter(
            *zip(*subtractions),
            marker=".",
            label="HyperLogLog correction",
            color=report.correction_hyperloglog.color,
        )

        relative_errors = []
        subtractions = []
        cardinalities_covered = set()

        for exact, estimated in zip(
            report.correction_hash_list.exact_cardinality_mean,
            report.correction_hash_list.estimated_cardinality_mean,
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
            color=report.hash_list_color,
        )
        axs[i][1].scatter(
            *zip(*subtractions),
            marker=".",
            label="HashList correction",
            color=report.hash_list_color,
        )

        expected_error = expected_hll_error(correction.precision)

        steps = np.array(
            [report.largest_exact_cardinality * step / 1000 for step in range(0, 1000)]
        )

        if 1 not in steps:
            steps = np.append(steps, 1)

        steps.sort()

        if report.largest_absolute_relative_error >= expected_hll_error(
            correction.precision
        ):
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

        axs[i][0].set_xlabel("Exact cardinality")
        axs[i][1].set_xlabel("Exact cardinality")
        axs[i][0].set_ylabel(f"Relative error ({scale})")
        axs[i][1].set_ylabel(f"Exact - Estimated ({scale})")
        axs[i][0].legend(ncol=2, loc="lower right")
        axs[i][1].legend(ncol=2, loc="lower right")
        axs[i][0].grid(which="both", linestyle="--", alpha=0.5)
        axs[i][1].grid(which="both", linestyle="--", alpha=0.5)
        axs[i][0].set_yscale(scale)
        axs[i][1].set_yscale(scale)
        axs[i][0].set_title(
            f"Relative errors p={correction.precision}, b={correction.bits}"
        )
        axs[i][1].set_title(
            f"Subtraction p={correction.precision}, b={correction.bits}"
        )

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
