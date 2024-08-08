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
    cardinality,HashSet,HLL6 + Siphasher13,0.00000,First,3809216.6544,0.0000000000000002220446049250313,0,3296,0.010823410313277677,0.008056849624719085,12,0.01625
    cardinality,HashSet,HLL6 + Siphasher24,0.00000,First,3809216.6544,0.0000000000000002220446049250313,0,3296,0.011627617049038684,0.008446708230154911,12,0.01625
    cardinality,HashSet,HLL6 + Xxhasher,0.00000,First,3809216.6544,0.0000000000000002220446049250313,0,3296,0.01238917485216544,0.008562564133659738,12,0.01625

    """
    paths = glob("tests/statistical_tests_reports/*.csv")
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


def load_results():
    """Load the results from the statistical tests.

    Implementative details
    ----------------------
    The header of the CSV files is as follows:

    feature,first_approach,second_approach,p-value,winner,first_mean,first_std,second_mean,second_std,precision,theoretical_error
    cardinality,Tabac's HLL++,Tabac's HLL,0.04494,First,0.01266,0.00961,0.01305,0.00982,12,0.01625
    cardinality,Tabac's HLL++,Streaming Algorithms,0.00000,Second,0.01266,0.00961,0.01089,0.00798,12,0.01625
    cardinality,Tabac's HLL++,HLL6 + Siphasher13,0.00000,Second,0.01266,0.00961,0.01082,0.00806,12,0.01625
    cardinality,Tabac's HLL++,HLL6 + Siphasher24,0.00001,Second,0.01266,0.00961,0.01163,0.00845,12,0.01625
    cardinality,Tabac's HLL++,HLL6 + Xxhasher,0.87753,None,0.01266,0.00961,0.01239,0.00856,12,0.01625
    cardinality,Tabac's HLL,Streaming Algorithms,0.00000,Second,0.01305,0.00982,0.01089,0.00798,12,0.01625
    cardinality,Tabac's HLL,HLL6 + Siphasher13,0.00000,Second,0.01305,0.00982,0.01082,0.00806,12,0.01625
    cardinality,Tabac's HLL,HLL6 + Siphasher24,0.00000,Second,0.01305,0.00982,0.01163,0.00845,12,0.01625
    cardinality,Tabac's HLL,HLL6 + Xxhasher,0.02789,Second,0.01305,0.00982,0.01239,0.00856,12,0.01625
    cardinality,Streaming Algorithms,HLL6 + Siphasher13,0.81066,None,0.01089,0.00798,0.01082,0.00806,12,0.01625
    cardinality,Streaming Algorithms,HLL6 + Siphasher24,0.00000,First,0.01089,0.00798,0.01163,0.00845,12,0.01625
    cardinality,Streaming Algorithms,HLL6 + Xxhasher,0.00000,First,0.01089,0.00798,0.01239,0.00856,12,0.01625
    cardinality,HLL6 + Siphasher13,HLL6 + Siphasher24,0.00000,First,0.01082,0.00806,0.01163,0.00845,12,0.01625
    cardinality,HLL6 + Siphasher13,HLL6 + Xxhasher,0.00000,First,0.01082,0.00806,0.01239,0.00856,12,0.01625
    cardinality,HLL6 + Siphasher24,HLL6 + Xxhasher,0.00019,First,0.01163,0.00845,0.01239,0.00856,12,0.01625

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


def is_our_approach(approach_name: str) -> bool:
    """Returns whether the approach is one of the approaches we have created."""
    return approach_name in [
        "HLL6 + Siphasher13",
        "HLL6 + Siphasher24",
        "HLL6 + Xxhasher",
        "HLL5 + Siphasher13",
        "HLL5 + Siphasher24",
        "HLL5 + Xxhasher",
        "HLL4 + Siphasher13",
        "HLL4 + Siphasher24",
        "HLL4 + Xxhasher",
        "MLE2 + Siphasher13",
        "MLE2 + Siphasher24",
        "MLE2 + Xxhasher",
        "MLE3 + Siphasher13",
        "MLE3 + Siphasher24",
        "MLE3 + Xxhasher",
    ]


def get_approach_color(approach_name: str) -> str:
    """Returns the color for a given normalized approach name."""
    return {
        "HashSet": "red",
        "HLL6 + Siphasher13": "tab:blue",
        "HLL6 + Siphasher24": "tab:purple",
        "HLL6 + Xxhasher": "tab:pink",
        "HLL5 + Siphasher13": "tab:blue",
        "HLL5 + Siphasher24": "tab:purple",
        "HLL5 + Xxhasher": "tab:pink",
        "HLL4 + Siphasher13": "tab:blue",
        "HLL4 + Siphasher24": "tab:purple",
        "HLL4 + Xxhasher": "tab:pink",
        "MLE2 + Siphasher13": "tab:blue",
        "MLE2 + Siphasher24": "tab:purple",
        "MLE2 + Xxhasher": "tab:pink",
        "MLE3 + Siphasher13": "tab:blue",
        "MLE3 + Siphasher24": "tab:purple",
        "MLE3 + Xxhasher": "tab:pink",
        "Tabac's HLL": "tab:red",
        "Tabac's HLL++": "tab:brown",
        "Streaming Algorithms": "tab:orange",
        "Theoretical (6 bits)": "green",
    }[approach_name]


def get_approach_linestyle(approach_name: str) -> str:
    """Returns the marker for a given normalized approach name."""
    return {
        "HashSet": ":",
        "HLL6 + Siphasher13": "-",
        "HLL6 + Siphasher24": "-",
        "HLL6 + Xxhasher": "-",
        "HLL5 + Siphasher13": "-",
        "HLL5 + Siphasher24": "-",
        "HLL5 + Xxhasher": "-",
        "HLL4 + Siphasher13": "-",
        "HLL4 + Siphasher24": "-",
        "HLL4 + Xxhasher": "-",
        "MLE2 + Siphasher13": "-",
        "MLE2 + Siphasher24": "-",
        "MLE2 + Xxhasher": "-",
        "MLE3 + Siphasher13": "-",
        "MLE3 + Siphasher24": "-",
        "MLE3 + Xxhasher": "-",
        "Tabac's HLL": "--",
        "Tabac's HLL++": "--",
        "Streaming Algorithms": "--",
        "Theoretical (6 bits)": "-.",
    }[approach_name]


def get_approach_marker(approach_name: str) -> str:
    """Returns the marker for a given normalized approach name."""
    return {
        "HashSet": "s",
        "HLL6 + Siphasher13": "$6$",
        "HLL6 + Siphasher24": "$6$",
        "HLL6 + Xxhasher": "$6$",
        "HLL5 + Siphasher13": "$5$",
        "HLL5 + Siphasher24": "$5$",
        "HLL5 + Xxhasher": "$5$",
        "HLL4 + Siphasher13": "$4$",
        "HLL4 + Siphasher24": "$4$",
        "HLL4 + Xxhasher": "$4$",
        "MLE2 + Siphasher13": "$M2$",
        "MLE2 + Siphasher24": "$M2$",
        "MLE2 + Xxhasher": "$M2$",
        "MLE3 + Siphasher13": "$M3$",
        "MLE3 + Siphasher24": "$M3$",
        "MLE3 + Xxhasher": "$M3$",
        "Tabac's HLL": "x",
        "Tabac's HLL++": "x",
        "Streaming Algorithms": "x",
        "Theoretical (6 bits)": ".",
    }[approach_name]


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

    fig.savefig(f"tests/statistical_tests_reports/{feature_name}.png")
    plt.close()


def get_win_tie_loss_table() -> pd.DataFrame:
    """Compose the win-tie-loss table for the approaches."""
    tests_df = load_test_results()

    # We drop the results associated to the HashSet approach.
    tests_df = tests_df[
        (tests_df["first_approach"] != "HashSet")
        & (tests_df["second_approach"] != "HashSet")
    ]

    outcomes = ["win", "tie", "loss"]

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

        results.extend(task_results)

    win_tie_loss_table = pd.DataFrame(results)

    # We sort by the number of wins.
    win_tie_loss_table = win_tie_loss_table.sort_values(
        by=["task", "win", "approach"], ascending=[True, False, False]
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
    win_tie_loss_table.to_csv("tests/statistical_tests_reports/win_tie_loss_table.csv")
    win_tie_loss_table.to_markdown(
        "tests/statistical_tests_reports/win_tie_loss_table.md", index=False
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


if __name__ == "__main__":
    plot_results(load_results())
