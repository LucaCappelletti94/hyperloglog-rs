# Optimal gap codes
A rayon-parallelized utility to determine which encoding is more space efficient for a given
precision, number of bits and hash size.

## How to run
As usual, just run:

```bash
RUSTFLAGS='-C target-cpu=native' cargo run --release
```

## Results
The results will be stored in the [`optimal_gap_codes.csv`]() file. Here is a preview of 
a few lines:

