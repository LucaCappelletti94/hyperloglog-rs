"""Script python to plot and compare the performance of the hybrid hashes with HLL"""

from typing import Optional, List, Dict, Tuple
import decimal
from decimal import Decimal
import matplotlib.pyplot as plt
from tqdm import tqdm


# Set precision high enough to handle large values
decimal.getcontext().prec = 1000


def compute_hll_error_rate(precision_bits: int) -> Decimal:
    """Computes the error rate of HyperLogLog for a given precision.
    
    The error rate is computed as 1.04 / sqrt(2^precision_bits).

    Parameters:
        precision_bits (int): Number of bits used for the precision of HyperLogLog.
    """
    # Number of registers m = 2^precision_bits
    m = Decimal(2**precision_bits)

    # Error rate for HyperLogLog
    error_rate = Decimal(1.04) / m.sqrt()

    return error_rate


def compute_expected_collisions(
    uniform_bits: int,           # Total bits in the hash
    geometric_bits: int, # Geometric bits (b)
    m_samples: int,        # Number of samples (total hashes)
    saturation: bool
) -> Decimal:
    """Computes the expected number of collisions for the hybrid hash."""
    if m_samples < 2:
        return 0
    
    # First, we need to determine the number of hashes that can be
    # stored in the provided hash size, considering that part of the
    # hash size is composed by the geometric bits. Furthermore, we also
    # need to consider that when the value stored in the geometric bits
    # is less or equal to the number of geometric bits, we will store it
    # using unary encoding, and use the newly freed bits to store more 
    # uniform bits, hence expanding the number of hashes that can be stored.

    # We determine the probability P(value of geometric bits <= geometric_bits)
    if saturation:
        p_geometric_bits = Decimal(0.0)
    else:
        uniform_bits = uniform_bits - 1
        p_geometric_bits = Decimal(geometric_bits + 1.0) / Decimal(2.0**geometric_bits)

    # The entropy provided by a geometric distribution with probability p = 1/2
    # is 2. Hence, we can store 2 bits in the geometric bits.

    # We weight the contributions of the possible values of the hash given the
    # case where:

    number_of_hashes = (
        # value of geometric bits <= geometric_bits
        Decimal(2**(uniform_bits + geometric_bits)) * p_geometric_bits +
        # value of geometric bits > geometric_bits
        Decimal(2**(uniform_bits + 2)) * (Decimal(1.0) - p_geometric_bits)
    )

    # We compute the expected number of collisions
    return Decimal(m_samples) * Decimal(m_samples - 1) / Decimal(2 * number_of_hashes)


def plot_errors():
    """Plots the expected error rate of HyperLogLog and the collision probability of the hybrid hash."""
    # Precision values for HyperLogLog
    precisions = list(range(4, 19))
    bits = [4, 5, 6]

    # We make two plots: one with the absolute errors, and the other one with the errors
    # normalized by the expected hyperloglog error rate
    fig, axes = plt.subplots(1, len(bits), figsize=(10 * len(bits), 10), dpi=100)

    # Compute the expected error rate of HyperLogLog
    hll_error_rates = [compute_hll_error_rate(precision) for precision in precisions]

    # Plot the expected error rate of HyperLogLog
    for ax in axes:
        ax.plot(
            precisions, hll_error_rates, label="HyperLogLog", color="blue", marker="o"
        )

    # We store the error rates for each combination of hash size, precisions and number of bits
    # so that we can afterwards plot for each bit size the smallest hash size that still beats
    # the HyperLogLog error rate
    error_rates: Dict[int, List[Tuple[float, int, int]]] = {}
    error_rates_best_non_zero: Dict[int, List[Tuple[float, int, int]]] = {}
    immediately_after_error_rates: Dict[int, List[Tuple[float, int, int]]] = {}

    # For each bit size
    for number_of_bits in (4, 5, 6):
        # We create a list with the error rate with smallest hash size
        # that still beats the HyperLogLog error rate
        bit_error_rates: List[Tuple[float, int, int]] = []
        bit_immediately_after_error_rates: List[Tuple[float, int, int]] = []
        bit_error_rates_best_non_zero: List[Tuple[float, int, int]] = []
        for precision in precisions:
            best_hash_size = 0
            best_error_rate = 1.0
            for hash_size in reversed(range(precision + number_of_bits, 33)):
                maximal_number_of_hashes = int(
                    float((2**precision) * number_of_bits) / float(hash_size)
                )
                expected_collisions = compute_expected_collisions(
                    uniform_bits=hash_size - number_of_bits,
                    geometric_bits=number_of_bits,
                    m_samples=maximal_number_of_hashes,
                    saturation=precision + number_of_bits == hash_size
                )
                error_rate = expected_collisions / maximal_number_of_hashes

                if error_rate > hll_error_rates[precision - 4] / 100 and len(bit_error_rates_best_non_zero) < precision - 3:
                    bit_error_rates_best_non_zero.append((error_rate, precision, hash_size))

                if error_rate >= hll_error_rates[precision - 4]:
                    bit_immediately_after_error_rates.append((error_rate, precision, hash_size))
                    break

                best_hash_size = hash_size
                best_error_rate = error_rate

            if best_error_rate < hll_error_rates[precision - 4]:
                bit_error_rates.append((best_error_rate, precision, best_hash_size))
        
        error_rates[number_of_bits] = bit_error_rates
        immediately_after_error_rates[number_of_bits] = bit_immediately_after_error_rates
        error_rates_best_non_zero[number_of_bits] = bit_error_rates_best_non_zero

    # Next, we plot the error rates for each bit size, including also
    # the annotation at each datapoint of which hash size is being used
    for (number_of_bits, ax) in zip(bits, axes):
        bit_error_rates = error_rates[number_of_bits]
        bit_immediately_after_error_rates = immediately_after_error_rates[number_of_bits]
        bit_error_rates_best_non_zero = error_rates_best_non_zero[number_of_bits]

        assert len(bit_immediately_after_error_rates) > 0

        ax.plot(
            [precision for _, precision, _ in bit_error_rates],
            [error_rate for error_rate, _, _ in bit_error_rates],
            label=f"Smallest viable hash (b={number_of_bits})",
            marker="o",
        )
        ax.plot(
            [precision for _, precision, _ in bit_immediately_after_error_rates],
            [error_rate for error_rate, _, _ in bit_immediately_after_error_rates],
            label=f"Immediately after (b={number_of_bits})",
            marker="o",
        )

        ax.plot(
            [precision for _, precision, _ in bit_error_rates_best_non_zero],
            [error_rate for error_rate, _, _ in bit_error_rates_best_non_zero],
            label=f"Largest sensible hash (b={number_of_bits})",
            marker="o",
        )

        for (error_rate, precision, hash_size) in bit_error_rates:
            ax.annotate(
                f"u{hash_size}",
                (precision, error_rate),
                textcoords="offset points",
                xytext=(0, 5),
                ha="center",
            )
        
        for (error_rate, precision, hash_size) in bit_immediately_after_error_rates:
            ax.annotate(
                f"u{hash_size}",
                (precision, error_rate),
                textcoords="offset points",
                xytext=(0, 5),
                ha="center",
            )

        for (error_rate, precision, hash_size) in bit_error_rates_best_non_zero:
            ax.annotate(
                f"u{hash_size}",
                (precision, error_rate),
                textcoords="offset points",
                xytext=(0, 5),
                ha="center",
            )

        # Set the title and labels
        ax.set_title("Error Rate vs Precision")
        ax.set_xlabel("Precision")
        ax.set_ylabel("Error Rate")
        ax.set_yscale("log")
        ax.legend()

        # Set the grid
        ax.grid(True, which="both", linestyle="--")

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
            uniform_bits=hash_size - 6,
            geometric_bits=6,
            m_samples=maximal_number_of_hashes,
            saturation=precision + 6 == hash_size
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
