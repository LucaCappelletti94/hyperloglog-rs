"""Script python to plot and compare the performance of the hybrid hashes with HLL"""

import matplotlib.pyplot as plt
import math
from typing import Optional
from tqdm import tqdm

import decimal
from decimal import Decimal

# Set precision high enough to handle large values
decimal.getcontext().prec = 1000


def compute_hll_error_rate(precision_bits: int) -> Decimal:
    # Number of registers m = 2^precision_bits
    m = Decimal(2**precision_bits)

    # Error rate for HyperLogLog
    error_rate = Decimal(1.04) / m.sqrt()

    return error_rate


def compute_expected_collisions(
    m_bits: int,           # Total bits in the hash
    n_uniform_bits: int,   # Number of strictly uniform bits (p)
    b_geometric_bits: int, # Geometric bits (b)
    m_samples: int,        # Number of samples (total hashes)
    p: Decimal = Decimal(0.5),  # Geometric distribution parameter (typically 0.5)
) -> Decimal:
    # Effective number of bits in the hash (excluding the flag bit)
    effective_bits = m_bits - 1
    
    # Maximum value 2^b - 1 (this is the maximum number of leading zeros that can be represented)
    max_leading_zeros = (2 ** b_geometric_bits) - 1
    
    # Number of non-uniform bits that remain after subtracting the always-uniform bits and the flag bit
    remaining_bits = effective_bits - n_uniform_bits
    
    # Calculate how many bits are taken based on the value of leading zeros
    # Case 1: 2^b <= remaining_bits (leading zeros and additional bits are used)
    # Here, all remaining bits are directly taken from the uniform hash
    n_total_bits_case1 = n_uniform_bits + remaining_bits
    # Case 2: 2^b > remaining_bits (write leading zeros using b bits, and remaining uniform bits)
    n_total_bits_case2 = n_uniform_bits + (remaining_bits - b_geometric_bits)
    
    # Probabilities for each case (geometric distribution PMF)
    P_case1 = Decimal(min(max_leading_zeros + 1, remaining_bits)) / (max_leading_zeros + 1)
    P_case2 = Decimal(1) - P_case1
    
    # Number of possible distinct values for case 1
    N_case1 = Decimal(2 ** n_total_bits_case1)
    
    # Number of possible distinct values for case 2
    N_case2 = Decimal(2 ** n_total_bits_case2)
    
    # Expected number of unique values for case 1
    expected_unique_case1 = N_case1 * (1 - (1 - Decimal(1) / N_case1) ** m_samples)
    
    # Expected number of unique values for case 2
    expected_unique_case2 = N_case2 * (1 - (1 - Decimal(1) / N_case2) ** m_samples)
    
    # Weighted average of expected unique values based on the probabilities
    expected_unique = (P_case1 * expected_unique_case1) + (P_case2 * expected_unique_case2)
    
    # Expected collisions is the total number of samples minus expected unique values
    expected_collisions = Decimal(m_samples) - expected_unique

    return expected_collisions


def plot_errors():
    """Plots the expected error rate of HyperLogLog and the collision probability of the hybrid hash."""
    # Precision values for HyperLogLog
    precisions = list(range(4, 19))

    # We make two plots: one with the absolute errors, and the other one with the errors
    # normalized by the expected hyperloglog error rate
    fig, axes = plt.subplots(1, 1, figsize=(10, 10), dpi=100)

    # Compute the expected error rate of HyperLogLog
    hll_error_rates = [compute_hll_error_rate(precision) for precision in precisions]

    # Plot the expected error rate of HyperLogLog
    axes.plot(
        precisions, hll_error_rates, label="HyperLogLog", color="blue", marker="o"
    )

    # Compute the collision probability of the hybrid hash
    for hash_size in (8, 16, 24, 32, 40, 48, 56, 64):
        for number_of_bits in (4, 5, 6):
            mean_collisions = []
            for precision in precisions:
                if number_of_bits + precision > hash_size:
                    continue
                maximal_number_of_hashes = int(
                    float((2**precision) * number_of_bits) / float(hash_size)
                )
                expected_collisions = compute_expected_collisions(
                    m_bits=hash_size,
                    n_uniform_bits=precision,
                    b_geometric_bits=number_of_bits,
                    m_samples=maximal_number_of_hashes,
                )
                mean_collisions.append(expected_collisions / maximal_number_of_hashes)

            # Plot the collision probability of the hybrid hash
            axes.plot(
                precisions[:len(mean_collisions)],
                mean_collisions,
                label=f"(m={hash_size}, b={number_of_bits})",
                marker="o",
            )

    # Set the title and labels
    axes.set_title("Error Rate vs Precision")
    axes.set_xlabel("Precision")
    axes.set_ylabel("Error Rate")
    axes.set_yscale("log")
    axes.legend(ncol=2)

    # Set the grid
    axes.grid(True, which="both", linestyle="--")

    fig.tight_layout()

    # Save the plot

    plt.savefig("scripts/error_rate_vs_collision_probability.png")


def hybrid_approach_absolute_error(cardinality: int, precision: int, bits: int, hyper_log_log_error_rate_at_precision: Decimal) -> (Decimal, bool, Optional[int]):
    """Computes the expected error of the Hybrid approach for a given precision.

    The Hybrid approach employes for a given cardinality the smallest hash size
    that will both fit the cardinality and the precision, while not surpassing
    the hyperloglog error rate, otherwise it will use the hyperloglog approach.

    The hash size employed must be a multiple of 8. While we use the provided bits
    to compute the number of bits that are in the data structure, we will strictly
    use 4 as number of geometrical bits we will use in the computation of the hash.
    """
    number_of_bits_in_data_structure = bits * 2.0**precision

    hash_size_identified = None

    # Compute the number of hashes that can be stored in the hash size
    for hash_size in (32, 24, 16, 8):
        maximal_number_of_hashes = int(
            float(number_of_bits_in_data_structure) / float(hash_size)
        )
        if maximal_number_of_hashes < cardinality:
            continue

        if precision + 4 > hash_size:
            continue

        expected_collisions = compute_expected_collisions(
            m_bits=hash_size,
            n_uniform_bits=precision,
            b_geometric_bits=6,
            m_samples=maximal_number_of_hashes
        )
        if expected_collisions / cardinality < hyper_log_log_error_rate_at_precision:
            hash_size_identified = hash_size
            break

    if hash_size_identified is None:
        return hyper_log_log_error_rate_at_precision * cardinality, True, hash_size_identified
    
    return max(expected_collisions, 1.0), False, hash_size_identified



def plot_absolute_error_by_cardinality():
    """Plots the absolute error of the Hybrid approach for different cardinalities.

    We consider cardinalities ranging from 0 to 1_000_000. We compute the expected
    error of the Hybrid approach for each cardinality.

    """

    # Precision values for HyperLogLog
    precisions = list(range(4, 19))

    # Compute the expected error rate of HyperLogLog
    hll_error_rates = [compute_hll_error_rate(precision) for precision in precisions]

    fig, axes = plt.subplots(1, 1, figsize=(10, 10), dpi=200)
    
    for precision in tqdm(precisions, leave=False, desc="Precision"):
        for bits in (6,):
            segments = []
            cardinality = 2
            current_hash_size = None
            error, uses_hll, hash_size= hybrid_approach_absolute_error(cardinality, precision, bits, hll_error_rates[precision - 4])
            
            cardinalities = [cardinality]
            hybrid_errors = [error]
            hash_changes = []

            while not uses_hll:
                cardinality *= 1.15
                error, uses_hll, hash_size= hybrid_approach_absolute_error(int(cardinality), precision, bits, hll_error_rates[precision - 4])

                if hash_size != current_hash_size:
                    current_hash_size = hash_size
                    segments.append((cardinalities, hybrid_errors))
                    cardinalities = []
                    hybrid_errors = []
                    if uses_hll:
                        hash_changes.append(("HLL", cardinality, error))
                    else:
                        hash_changes.append((f"u{hash_size}", cardinality, error))

                cardinalities.append(cardinality)
                hybrid_errors.append(error)

            while cardinality < 75_000:
                cardinality *= 1.5
                error, _, _= hybrid_approach_absolute_error(int(cardinality), precision, bits, hll_error_rates[precision - 4])
                cardinalities.append(cardinality)
                hybrid_errors.append(error)

            segments.append((cardinalities, hybrid_errors))

            color = None

            last_point = None
            for (cardinalities, hybrid_errors) in segments:
                line = axes.plot(
                    cardinalities,
                    hybrid_errors,
                    color=color,
                    marker=".",
                    label=f"(p={precision}, b={bits})" if last_point is None else None,
                )
                color = line[0].get_color()
                if last_point is not None:
                    axes.plot(
                        [last_point[0], cardinalities[0]],
                        [last_point[1], hybrid_errors[0]],
                        color=color,
                        linestyle="--"
                    )

                last_point = (cardinalities[-1], hybrid_errors[-1])

            for hash_change in hash_changes:
                axes.annotate(
                    hash_change[0],
                    (hash_change[1], hash_change[2]),
                    textcoords="offset points",
                    xytext=(0, 5),
                    color=line[0].get_color(),
                    ha="center",
                )

    # Set the title and labels
    axes.set_title("Absolute Error vs Cardinality")
    axes.set_xlabel("Cardinality")
    axes.set_ylabel("Absolute Error")
    axes.set_yscale("log")
    axes.set_xscale("log")
    axes.legend(ncol=2)

    # Set the grid
    axes.grid(True, which="both", linestyle="--")

    fig.tight_layout()

    # Save the plot
    plt.savefig("scripts/absolute_error_vs_cardinality.png")

if __name__ == "__main__":
    plot_errors()
    plot_absolute_error_by_cardinality()
