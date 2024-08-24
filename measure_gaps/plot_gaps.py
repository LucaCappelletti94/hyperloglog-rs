"""Plot histograms of the hash gaps.

This script, for each precision considered in the analysis, i.e. the range 4 to 18,
loads the gzipped CSVs from the reports directory and plots the histograms of the gap counts
for each hash size and each bit size, and hasher used.

The names of the documents are of the format 'reports/gap_report_precision_{}_bits_{}_hash_{}_{}.csv.gz'
where the first placeholder is the precision, the second is the bit size, the third is the hash size,
and the fourth is the hasher used.

The layour expected is a grid with "number of hash sizes" columns and "number of bit sizes" rows.
Different precisions and different hasher are plotted in different figures.

"""

import os
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
from tqdm.auto import trange, tqdm


def plot(df: pd.DataFrame, precision: int, hasher: str):
    """Plot the histograms of the gap counts.
    
    Parameters
    ----------
    df : pd.DataFrame
        The DataFrame containing the gap counts.
    precision : int
        The precision used in the analysis.
    hasher : str
        The hasher used in the analysis
    """

    number_of_unique_hashes = df["hash_size"].nunique()
    number_of_unique_bits = df["bit_size"].nunique()

    fig, axes = plt.subplots(
        number_of_unique_bits,
        number_of_unique_hashes,
        figsize=(5 * number_of_unique_hashes, 5 * number_of_unique_bits),
    )

    for i, hash_size in enumerate(df["hash_size"].unique()):
        for j, bit_size in enumerate(df["bit_size"].unique()):
            data = df[(df["hash_size"] == hash_size) & (df["bit_size"] == bit_size)]
            ax = axes[j, i]

            if data.empty:
                # If the data is empty, we draw a red X in the plot
                ax.text(0.5, 0.5, "X", fontsize=24, color="red", ha="center", va="center")
                ax.axis("off")
                continue
                
            ax.set_title(f"Hash size: {hash_size}, Bit size: {bit_size}")
            sns.histplot(
                data,
                x="gap",
                ax=ax,
                bins=500,
            )

    fig.suptitle(f"Precision: {precision}, Hasher: {hasher}")

    plt.tight_layout()

    plt.savefig(f"reports/gap_report_precision_{precision}_{hasher}.png")


def plot_all():
    """Plot all the histograms of the gap counts."""
    hasher = "xxhash64"
    sns.set_style("whitegrid")
    for precision in trange(4, 19, desc="Precision"):
        dataframes = []
        for hash_size in tqdm([1, 2, 3, 4], desc="Hash size", leave=False):
            for bit_size in tqdm([4, 5, 6], desc="Bit size", leave=False):
                path = f"reports/gap_report_precision_{precision}_bits_{bit_size}_hash_{hash_size}_{hasher}.csv.gz"
                if os.path.exists(path):
                    df = pd.read_csv(
                        path,
                        compression="gzip",
                        dtype={"gap": "int32", "count": "int64"},
                    )
                    df["hash_size"] = hash_size
                    df["bit_size"] = bit_size
                    dataframes.append(df)

        df = pd.concat(dataframes)

        plot(df, precision, hasher)


if __name__ == "__main__":
    plot_all()
