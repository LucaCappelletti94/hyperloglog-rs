"""Python script to plot the results obtained from the statistical tests."""

from glob import glob
import matplotlib.pyplot as plt
import pandas as pd
from tqdm.auto import tqdm


def load_test_results():
    """Load the results from the statistical tests.

    Implementative details
    ----------------------
    The header of the CSV files is as follows:

    feature,first_approach,second_approach,p-value,winner,first_memsize,first_mean,first_std,second_memsize,second_mean,second_std,precision,theoretical_error
    cardinality,HashSet,Tabac's HLL++,0.00000,First,3809216.6544,0.0000000000000002220446049250313,0,3474.256,0.01265437523044048,0.009503921612939932,12,0.01625
    cardinality,HashSet,Tabac's HLL,0.00000,First,3809216.6544,0.0000000000000002220446049250313,0,2804,0.012661636088409827,0.009466995766800382,12,0.01625
    cardinality,HashSet,Streaming Algorithms,0.00000,First,3809216.6544,0.0000000000000002220446049250313,0,4152,0.010886160965328712,0.007983075620151478,12,0.01625
    cardinality,HashSet,HLL6 + WyHash,0.00000,First,3809216.6544,0.0000000000000002220446049250313,0,3296,0.010823410313277677,0.008056849624719085,12,0.01625
    cardinality,HashSet,HLL6 + DefaultHasher,0.00000,First,3809216.6544,0.0000000000000002220446049250313,0,3296,0.011627617049038684,0.008446708230154911,12,0.01625
    cardinality,HashSet,HLL6 + Xxhasher,0.00000,First,3809216.6544,0.0000000000000002220446049250313,0,3296,0.01238917485216544,0.008562564133659738,12,0.01625

    """
    paths = [
        path
        for path in glob("./statistical_tests_reports/*.csv")
        if path.count("_") == 3
    ]

    assert len(paths) > 0, "No results found."

    return pd.concat(
        [
            pd.read_csv(path)
            for path in tqdm(
                paths,
                desc="Loading results",
                unit="file",
                leave=False,
                dynamic_ncols=True,
            )
        ],
        ignore_index=True,
    )


def load_features(
    precision: int, feature_name: str
) -> (pd.DataFrame, pd.DataFrame, pd.DataFrame):
    """Returns a triple of dataframes with the feature data, the absolute errors and the memory requirements respectively.

    Parameters
    ----------
    precision : int
        The precision of the feature.
    feature_name : str
        The name of the feature.
    """
    feature_path = (
        f"./statistical_tests_reports/{feature_name}_features_{precision}.csv"
    )
    errors_path = (
        f"./statistical_tests_reports/{feature_name}_absolute_errors_{precision}.csv"
    )
    memory_path = f"./statistical_tests_reports/{feature_name}_memory_requirements_{precision}.csv"

    feature_df = pd.read_csv(feature_path, low_memory=False)
    errors_df = pd.read_csv(errors_path, low_memory=False)
    memory_df = pd.read_csv(memory_path, low_memory=False)

    # We exclude the approaches that include 'HLL8' or 'Beta8' in their names
    # as the error rate is identical to 'HLL6' and 'Beta6' and only increase
    # the noise in the plots.
    columns_to_drop = [
        column for column in feature_df.columns if "HLL8" in column or "Beta8" in column
    ]
    feature_df = feature_df.drop(columns=columns_to_drop)
    errors_df = errors_df.drop(columns=columns_to_drop)
    memory_df = memory_df.drop(columns=columns_to_drop)

    # We sort all thress dataframes by the feature's column 'HashSet',
    # which determines the exact cardinality of the specific run.
    # This column is only present in the feature dataframe.
    errors_df["column_to_sort"] = feature_df["HashSet"]
    memory_df["column_to_sort"] = feature_df["HashSet"]

    feature_df = feature_df.sort_values(by="HashSet")
    errors_df = errors_df.sort_values(by="column_to_sort")
    memory_df = memory_df.sort_values(by="column_to_sort")

    # We drop the column 'column_to_sort' from the dataframes.
    errors_df = errors_df.drop(columns=["column_to_sort"])
    memory_df = memory_df.drop(columns=["column_to_sort"])

    # We smooth the values of all the columns in the error dataframe
    # so to remove some noise from the measurements.
    errors_df = errors_df.rolling(window=300, closed="both").mean()

    return feature_df, errors_df, memory_df


def load_results():
    """Load the results from the statistical tests.

    Implementative details
    ----------------------
    The header of the CSV files is as follows:

    feature,first_approach,second_approach,p-value,winner,first_mean,first_std,second_mean,second_std,precision,theoretical_error
    cardinality,Tabac's HLL++,Tabac's HLL,0.04494,First,0.01266,0.00961,0.01305,0.00982,12,0.01625
    cardinality,Tabac's HLL++,Streaming Algorithms,0.00000,Second,0.01266,0.00961,0.01089,0.00798,12,0.01625
    cardinality,Tabac's HLL++,HLL6 + WyHash,0.00000,Second,0.01266,0.00961,0.01082,0.00806,12,0.01625
    cardinality,Tabac's HLL++,HLL6 + DefaultHasher,0.00001,Second,0.01266,0.00961,0.01163,0.00845,12,0.01625
    cardinality,Tabac's HLL++,HLL6 + Xxhasher,0.87753,None,0.01266,0.00961,0.01239,0.00856,12,0.01625
    cardinality,Tabac's HLL,Streaming Algorithms,0.00000,Second,0.01305,0.00982,0.01089,0.00798,12,0.01625
    cardinality,Tabac's HLL,HLL6 + WyHash,0.00000,Second,0.01305,0.00982,0.01082,0.00806,12,0.01625
    cardinality,Tabac's HLL,HLL6 + DefaultHasher,0.00000,Second,0.01305,0.00982,0.01163,0.00845,12,0.01625
    cardinality,Tabac's HLL,HLL6 + Xxhasher,0.02789,Second,0.01305,0.00982,0.01239,0.00856,12,0.01625
    cardinality,Streaming Algorithms,HLL6 + WyHash,0.81066,None,0.01089,0.00798,0.01082,0.00806,12,0.01625
    cardinality,Streaming Algorithms,HLL6 + DefaultHasher,0.00000,First,0.01089,0.00798,0.01163,0.00845,12,0.01625
    cardinality,Streaming Algorithms,HLL6 + Xxhasher,0.00000,First,0.01089,0.00798,0.01239,0.00856,12,0.01625
    cardinality,HLL6 + WyHash,HLL6 + DefaultHasher,0.00000,First,0.01082,0.00806,0.01163,0.00845,12,0.01625
    cardinality,HLL6 + WyHash,HLL6 + Xxhasher,0.00000,First,0.01082,0.00806,0.01239,0.00856,12,0.01625
    cardinality,HLL6 + DefaultHasher,HLL6 + Xxhasher,0.00019,First,0.01163,0.00845,0.01239,0.00856,12,0.01625

    """
    df = load_test_results()

    # We apply normalizations to the dataframe. As it is stored, it shows
    # the tuple for the results of statistical tests, but we want to plot
    # the mean and standard deviations of each approach.

    firsts = df[
        [
            "feature",
            "first_memsize",
            "first_approach",
            "first_mean",
            "first_std",
            "precision",
        ]
    ].copy()
    # We rename the 'first_mean' and 'first_std' columns to 'mean' and 'std'.
    firsts.columns = [
        "feature",
        "memory_requirements",
        "approach",
        "mean",
        "std",
        "precision",
    ]

    seconds = df[
        [
            "feature",
            "second_memsize",
            "second_approach",
            "second_mean",
            "second_std",
            "precision",
        ]
    ].copy()
    # We rename the 'second_mean' and 'second_std' columns to 'mean' and 'std', plus
    # we rename the 'approach' column to 'approach'.
    seconds.columns = [
        "feature",
        "memory_requirements",
        "approach",
        "mean",
        "std",
        "precision",
    ]

    theoretical_error = df[["feature", "precision", "theoretical_error"]].copy()
    # We rename the 'theoretical_error' column to 'mean', and add a new column
    # 'std' with value 0. Plus, we add a new column 'approach' with value 'Theoretical'.

    theoretical_error.columns = ["feature", "precision", "mean"]
    theoretical_error["std"] = 0
    theoretical_error["memory_requirements"] = [
        (2**precision) * 6 / 8 for precision in theoretical_error["precision"]
    ]
    theoretical_error["approach"] = "Theoretical (6 bits)"

    # We concatenate the three dataframes.
    df = pd.concat([theoretical_error, firsts, seconds], ignore_index=True)

    # We drop the duplicates as defined by the columns 'feature', 'approach', and 'precision'.
    df = df.drop_duplicates(subset=["feature", "approach", "precision"])

    # We adjust the standard deviation so that when mean - std < 0, std = mean.
    df["std"] = df[["mean", "std"]].min(axis=1)

    # We sort by memory requirements.
    df = df.sort_values(by="memory_requirements")

    return df

def cutoff_linear_counting(precision: int) -> int:
    """Return the cutoff for linear counting for a given precision."""
    return [
        10, 20, 40, 80, 220, 400, 900, 1_800, 3_100, 6_500, 11_500, 20_000, 50_000, 120_000,
        350_000,
    ][precision - 4]

def cutoff_bias_correction(precision: int) -> int:
    """Return the cutoff for bias correction for a given precision."""
    return 5 * (2 ** precision)

def is_our_approach(approach_name: str) -> bool:
    """Returns whether the approach is one of the approaches we have created."""
    needles = [
        "Tabac's HLL++",
        "Tabac's HLL",
        "Streaming Algorithms",
        "Cardinality Estimator",
        "Theoretical"
    ]

    return not any(needle in approach_name for needle in needles)


def get_approach_color(approach_name: str) -> str:
    """Returns the color for a given normalized approach name."""
    colors = pd.read_csv("../utilities/colors.csv", index_col="approach")
    return colors.loc[approach_name, "color"]


def get_approach_linestyle(approach_name: str) -> str:
    """Returns the marker for a given normalized approach name."""
    if is_our_approach(approach_name):
        return "-"
    return "--"


def get_approach_marker(approach_name: str) -> str:
    """Returns the marker for a given normalized approach name."""
    markers = pd.read_csv("../utilities/markers.csv", index_col="approach")
    return markers.loc[approach_name, "marker"]


def plot_feature(feature_name: str, feature_results: pd.DataFrame):
    """Plot the results obtained from the statistical tests for a feature."""

    # We plot each feature both in log scale and in linear scale.
    # The vertical axis is always in log scale as it represents the
    # rough estimate memory requirements.

    fig, axes = plt.subplots(2, 1, figsize=(15, 10), dpi=300)

    for i, yscale in enumerate(["linear", "log"]):
        xscale = "log"
        ax = axes[i]
        for approach_name, approach_results in feature_results.groupby("approach"):
            if yscale == "log" and approach_name == "HashSet":
                continue

            # We plot an area that represents the standard deviation.
            # ax.fill_between(
            #     approach_results["memory_requirements"],
            #     approach_results["mean"] - approach_results["std"],
            #     approach_results["mean"] + approach_results["std"],
            #     color=get_approach_color(approach_name),
            #     alpha=0.1,
            # )

            # ax.errorbar(
            #     approach_results["memory_requirements"],
            #     approach_results["mean"],
            #     linestyle=get_approach_linestyle(approach_name),
            #     yerr=approach_results["std"],
            #     color=get_approach_color(approach_name),
            #     label=approach_name,
            # )

            ax.plot(
                approach_results["memory_requirements"],
                approach_results["mean"],
                linestyle=get_approach_linestyle(approach_name),
                marker=get_approach_marker(approach_name),
                color=get_approach_color(approach_name),
                label=approach_name,
            )

        ax.set_ylabel(f"Mean absolute error ({yscale} scale)")
        ax.set_xlabel(f"Memory requirements (bytes, {xscale} scale)")
        ax.set_title(f"{feature_name}")
        ax.set_xscale(xscale)
        ax.set_yscale(yscale)
        ax.grid(True, which="both", ls="--")

        ax.legend(
            title="Library",
            title_fontsize="small",
            fontsize="small",
            loc="upper right",
        )

    fig.tight_layout()

    fig.savefig(f"./statistical_tests_reports/{feature_name}.png")
    plt.close()


def get_win_tie_loss_table() -> pd.DataFrame:
    """Compose the win-tie-loss table for the approaches."""
    tests_df = load_test_results()

    # We drop the results associated to the HashSet approach.
    tests_df = tests_df[
        (tests_df["first_approach"] != "HashSet")
        & (tests_df["second_approach"] != "HashSet")
    ]

    outcomes = ["win", "tie", "loss", "error"]

    results = []

    # For each task
    for task_name, task_df in tests_df.groupby("feature"):
        methods = {
            approach
            for approaches in [
                task_df["first_approach"].unique(),
                task_df["second_approach"].unique(),
            ]
            for approach in approaches
            if pd.notna(approach)
        }

        task_results = {
            method: {outcome: 0 for outcome in outcomes} for method in methods
        }

        # For each pair of methods
        for _, row in task_df.iterrows():
            first_method = row["first_approach"]
            second_method = row["second_approach"]
            task_results[first_method]["error"] += row["first_mean"]
            task_results[second_method]["error"] += row["second_mean"]

            if row["winner"] == "First":
                task_results[first_method]["win"] += 1
                task_results[second_method]["loss"] += 1
            elif row["winner"] == "Second":
                task_results[first_method]["loss"] += 1
                task_results[second_method]["win"] += 1
            else:
                task_results[first_method]["tie"] += 1
                task_results[second_method]["tie"] += 1

        # We flatten the dictionary to a list of dictionaries.
        task_results = [
            {"task": task_name, "approach": method, **outcomes}
            for method, outcomes in task_results.items()
        ]

        # We compute the mean of the error
        for result in task_results:
            result["error"] /= float(result["tie"] + result["win"] + result["loss"])

        results.extend(task_results)

    win_tie_loss_table = pd.DataFrame(results)

    # We sort by the number of wins.
    win_tie_loss_table = win_tie_loss_table.sort_values(
        by=["task", "error", "approach"], ascending=[True, True, False]
    )

    # Then we reset the index
    win_tie_loss_table = win_tie_loss_table.reset_index(drop=True)

    return win_tie_loss_table


def plot_results(results: pd.DataFrame):
    """Plot the results obtained from the statistical tests."""

    # We get the number of unique features in the dataset.
    n_features = results["feature"].nunique()
    win_tie_loss_table = get_win_tie_loss_table()

    # We write the win-tie-loss table to a CSV file and a markdown file.
    win_tie_loss_table.to_csv("./statistical_tests_reports/win_tie_loss_table.csv")
    win_tie_loss_table.to_markdown(
        "./statistical_tests_reports/win_tie_loss_table.md", index=False
    )

    # For each feature, we plot it in a separate figure.
    for feature_name, feature_results in tqdm(
        results.groupby("feature"),
        desc="Plotting results",
        total=n_features,
        unit="feature",
        leave=False,
        dynamic_ncols=True,
    ):
        plot_feature(feature_name, feature_results)

        # We identify which of the approaches that we have created
        # performs the best as determined by the number of wins from the statistical tests.
        win_tie_loss_table_feature = win_tie_loss_table[
            win_tie_loss_table["task"] == feature_name
        ]

        winning_approach = None
        # We iterate the first winning approach that is ours.
        for _, row in win_tie_loss_table_feature.iterrows():
            if is_our_approach(row["approach"]):
                winning_approach = row["approach"]
                break

        assert (
            winning_approach is not None
        ), f"No winning approach found for {feature_name}."

        # Then, we filter out all other non-winning approaches of ours, and plot
        # the results of the winning approach against the competing approaches.

        mask = [
            approach
            for approach in feature_results["approach"].unique()
            if approach == winning_approach or not is_our_approach(approach)
        ]

        feature_results = feature_results[feature_results["approach"].isin(mask)]

        plot_feature(f"{feature_name}_best", feature_results)


def theoretical_error(precision: int) -> float:
    """Return the theoretical error for a given precision."""
    return 1.04 / (2.0 ** (precision / 2))


def plot_measurements(
    feature_df: pd.DataFrame,
    values_df: pd.DataFrame,
    name_of_values: str,
    name_of_feature: str,
    precision: int,
):
    """Plot the results obtained from the statistical tests for a feature."""

    fig, axes = plt.subplots(2, 2, figsize=(20, 10), dpi=100)

    all_approaches_but_hashset = [
        approach for approach in values_df.columns if approach != "HashSet"
    ]

    values_df = values_df[all_approaches_but_hashset]

    for j, xscale in tqdm(
        enumerate(["linear", "log"]),
        total=2,
        desc="Plotting x measurements",
        leave=False,
    ):
        mean = values_df.mean().mean()
        for i, filter_mean in tqdm(
            enumerate([None, mean]),
            total=2,
            leave=False,
        ):
            for approach in values_df.columns:
                if filter_mean is not None and values_df[approach].mean() > filter_mean:
                    continue

                ax = axes[i, j]
                ax.plot(
                    feature_df["HashSet"],
                    values_df[approach],
                    color=get_approach_color(approach),
                    # marker=get_approach_marker(approach),
                    linestyle=get_approach_linestyle(approach),
                    label=approach,
                )

            # We also plot an horizontal line representing the theoretical error.
            ax.axhline(
                y=theoretical_error(precision),
                color="red",
                linestyle="--",
                label="Theoretical Error",
            )

            # We plot a vertical line showing the cutoff for linear counting.
            ax.axvline(
                x=cutoff_linear_counting(precision),
                color="green",
                linestyle="--",
                label="Linear Counting Cutoff",
            )

            # We plot a vertical line showing the cutoff for bias correction.
            ax.axvline(
                x=cutoff_bias_correction(precision),
                color="blue",
                linestyle="--",
                label="Bias Correction Cutoff",
            )

            ax.set_ylabel(f"{name_of_values} (log scale)")
            ax.set_xlabel(f"{name_of_feature} ({xscale} scale)")
            if filter_mean is not None:
                ax.set_title(
                    f"{name_of_feature} - Precision {precision} - Threshold {filter_mean:.2e}"
                )
            else:
                ax.set_title(f"{name_of_feature} - Precision {precision}")
            ax.set_yscale("log")
            ax.set_xscale(xscale)
            ax.grid(True, which="both", ls="--")

    axes[0, 0].legend(
        title="Library",
        title_fontsize="small",
        fontsize="small",
        loc="upper right",
    )

    fig.tight_layout()

    name_of_values = name_of_values.replace(" ", "_").lower()
    name_of_feature = name_of_feature.replace(" ", "_").lower()
    fig.savefig(
        f"./statistical_tests_reports/{name_of_values}_{name_of_feature}_{precision}.png"
    )
    plt.close()


def plot_all_measurements():
    """Plot the results obtained from the statistical tests for all features."""
    for feature_name in tqdm(
        ["cardinality", "union"], desc="Plotting measurements", leave=False
    ):
        for precision in tqdm(
            range(4, 19),
            total=19 - 4,
            desc=f"Plotting measurements of {feature_name}",
            leave=False,
        ):
            try:
                feature_df, errors_df, memory_df = load_features(
                    precision, feature_name
                )
            except FileNotFoundError:
                print(f"Report for precision {precision} not found.")
                continue

            plot_measurements(
                feature_df,
                errors_df,
                "Absolute Error",
                feature_name.capitalize(),
                precision,
            )
            # plot_measurements(feature_df, memory_df, "Memory Requirements", feature_name.capitalize(), precision)


if __name__ == "__main__":
    plot_results(load_results())
    plot_all_measurements()
