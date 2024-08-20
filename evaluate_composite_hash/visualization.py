"""Python script to visualize the collision rates of different composite hashes.

We plot the number of elements on the abscissa and the collision rate on the ordinate.
We split the report into different plots, where in each plot we display a different composite hash.
In each plot, we display the performance for all combinations of precision and number of bits.
We distinguish the precisions (4-18) by color, and the number of bits (4, 5, 6) by the style of the line
and of the marker. 

The CSV report, for reference, looks like this:

| hll_error           | mean_collision_rate      | mean_number_of_collisions | number_of_elements | number_of_bits | exponent | number_of_bits_composite_hash |
|---------------------|--------------------------|---------------------------|--------------------|----------------|----------|------------------------------|
| 0.26                | 0.0690105                | 0.552084                  | 8                  | 4              | 4        | 8                            |
| 0.26                | 0.0001165                | 0.000466                  | 4                  | 4              | 4        | 16                           |
| 0.26                | 0.0003196                | 0.001598                  | 5                  | 5              | 4        | 16                           |
| 0.26                | 0.0008173333333333332    | 0.004904                  | 6                  | 6              | 4        | 16                           |
| 0.18384776310850234 | 0.000277                 | 0.002216                  | 8                  | 4              | 5        | 16                           |
| 0.18384776310850234 | 0.0007198                | 0.007198                  | 10                 | 5              | 5        | 16                           |
| 0.18384776310850234 | 0.0017856666666666665    | 0.021428                  | 12                 | 6              | 5        | 16                           |


"""

import matplotlib.pyplot as plt
import pandas as pd


def align_hashes(df) -> pd.DataFrame:
    """Aligns and infers the performance of different composite hashes by the number of elements."""
    unique_number_of_elements = df["number_of_elements"].unique()
    unique_hash_values = df["number_of_bits_composite_hash"].unique()

    # We increase the minimum mean_collision_rate to avoid zeros to the next non-zero value
    df["mean_collision_rate"] = df["mean_collision_rate"].apply(
        lambda x: 1e-20 if x == 0 else x
    )

    # We combine the number of bits and the exponent to create a unique identifier which we will plot.
    df["hll"] = [
        f"P{precision}B{bits}"
        for bits, precision in zip(df["number_of_bits"], df["exponent"])
    ]

    # We determine for precision the maximal number of elements it can handle
    max_number_of_elements_per_hll = df.groupby(["hll"])["number_of_elements"].max()
    min_number_of_elements_per_hll_and_hash = df.groupby(
        ["hll", "number_of_bits_composite_hash"]
    )["number_of_elements"].min()

    # We impute the performance of missing unique number of elements for each object
    # were they are missing by putting the HLL expected error rate.
    missing_values = []
    for hll, max_number_of_elements in max_number_of_elements_per_hll.items():
        # We get the precision associated at the current hll object
        precision = int(hll.split("B")[0].split("P")[1])
        bits = int(hll.split("B")[1])
        hll_error = max(1.04 * (2 ** (-precision / 2.0)), 1e-20)

        for number_of_elements in unique_number_of_elements:
            if number_of_elements > max_number_of_elements:
                # This applies to all hashes.
                for unique_hash_value in unique_hash_values:
                    # We verify that there is indeed at least an entry for this hash
                    # associated with the current hll object, so not to display values
                    # for hashes that are not compatible at all with a given combination
                    # of precision and bits.
                    if df[
                        (df["hll"] == hll)
                        & (df["number_of_bits_composite_hash"] == unique_hash_value)
                    ].empty:
                        continue

                    missing_values.append(
                        {
                            "hll": hll,
                            "hll_error": hll_error,
                            "mean_collision_rate": hll_error,
                            "mean_number_of_collisions": hll_error * (max_number_of_elements + 1),
                            "number_of_elements": max_number_of_elements + 1,
                            "number_of_bits": bits,
                            "exponent": precision,
                            "number_of_bits_composite_hash": unique_hash_value,
                        }
                    )
                    missing_values.append(
                        {
                            "hll": hll,
                            "hll_error": hll_error,
                            "mean_collision_rate": hll_error,
                            "mean_number_of_collisions": hll_error * number_of_elements,
                            "number_of_elements": number_of_elements,
                            "number_of_bits": bits,
                            "exponent": precision,
                            "number_of_bits_composite_hash": unique_hash_value,
                        }
                    )
    # We do a similar procedure to infer the case where the number of elements is below the minimum
    for (
        hll,
        unique_hash_value,
    ), min_number_of_elements in min_number_of_elements_per_hll_and_hash.items():
        for number_of_elements in unique_number_of_elements:
            if number_of_elements < min_number_of_elements:
                # We retrieve the row associated to the current hash, hll and min number of elements
                # and we impute the missing values. If the row does not exist, as in some cases the
                # hash function is not compatible with precision and bit size, we skip it.
                row = df[
                    (df["hll"] == hll)
                    & (df["number_of_elements"] == min_number_of_elements)
                    & (df["number_of_bits_composite_hash"] == unique_hash_value)
                ].iloc[0]

                missing_values.append(
                    {
                        "hll": hll,
                        "hll_error": row.hll_error,
                        "mean_collision_rate": row.mean_collision_rate * (min_number_of_elements - 1) / min_number_of_elements,
                        "mean_number_of_collisions": row.mean_number_of_collisions,
                        "number_of_elements": min_number_of_elements - 1,
                        "number_of_bits": row.number_of_bits,
                        "exponent": row.exponent,
                        "number_of_bits_composite_hash": row.number_of_bits_composite_hash,
                    }
                )
                
                missing_values.append(
                    {
                        "hll": hll,
                        "hll_error": row.hll_error,
                        "mean_collision_rate": row.mean_collision_rate * number_of_elements / min_number_of_elements,
                        "mean_number_of_collisions": row.mean_number_of_collisions,
                        "number_of_elements": number_of_elements,
                        "number_of_bits": row.number_of_bits,
                        "exponent": row.exponent,
                        "number_of_bits_composite_hash": row.number_of_bits_composite_hash,
                    }
                )

    print(f"Imputed {len(missing_values)} missing values.")
    
    return pd.concat([df, pd.DataFrame(missing_values)])

def plot():
    """Plot the collision rates of different composite hashes."""
    # Load the collision rates of different composite hashes
    df = pd.read_csv("collision_rates.csv")

    # We only keep the case at 4 bits because it is the only one we can compare
    # across all hashes.
    df = df[df["number_of_bits"] == 4]

    # We align the composite hashes performance by the number of elements
    df = align_hashes(df)

    # We need to plot a total of 8 hashes. We place these subplots in a 4x2 grid.
    fig, axs = plt.subplots(2, 4, figsize=(20, 10), sharex=True, sharey=True)

    # We iterate over the different hashes and we plot them in the corresponding subplot
    unique_hashes = df["number_of_bits_composite_hash"].unique()

    colors_per_precision = {
        4: "tab:blue",
        5: "tab:orange",
        6: "tab:green",
        7: "tab:red",
        8: "tab:purple",
        9: "tab:brown",
        10: "tab:pink",
        11: "tab:gray",
        12: "tab:olive",
        13: "tab:cyan",
        14: "#FFA07A",
        15: "#FFD700",
        16: "#FF6347",
        17: "#FF4500",
        18: "#FF1493",
    }

    marker_per_bit_size = {
        4: "o",
        5: "s",
        6: "^",
    }

    line_style_per_bit_size = {
        4: "-",
        5: "--",
        6: "-.",
    }

    for i, hash_value in enumerate(unique_hashes):
        ax = axs[i // 4, i % 4]

        # We filter the data for the current hash
        hash_df = df[df["number_of_bits_composite_hash"] == hash_value]

        assert not hash_df.empty, f"No data for hash {hash_value}"

        # We iterate the hll objects and we plot them in the corresponding subplot
        for hll, group in hash_df.groupby("hll"):
            precision = int(hll.split("P")[1].split("B")[0])
            bits = int(hll.split("B")[1])

            # We sort the values by mean collision rate
            group = group.sort_values("number_of_elements")

            # We plot the linear scale
            ax.plot(
                group["number_of_elements"],
                group["mean_collision_rate"],
                label=f"P{precision}B{bits}",
                color=colors_per_precision[precision],
                marker=marker_per_bit_size[bits],
                markersize=1,
                linewidth=1,
                linestyle=line_style_per_bit_size[bits],
            )
        
        # We display the grids for the subplots
        ax.grid(True, which="both")

        # We display the legend for these two subplots, splitting
        # into 3 columns to avoid overlapping.
        ax.legend(ncol=3, fontsize="small")

        # We set the title for the linear scale
        ax.set_title(f"Composite Hash {hash_value} (Log Scale)")

        # We set the x-axis label for the linear scale
        ax.set_xlabel("Number of Elements (Log Scale)")
        ax.set_xscale("log")
        ax.set_yscale("log")

        # We set the y-axis label for the linear scale
        ax.set_ylabel("Collision Rate (Log Scale)")


    # We adjust the layout of the subplots
    fig.tight_layout()

    # We save the plot
    fig.savefig("collision_rates.png")


if __name__ == "__main__":
    plot()