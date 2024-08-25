# Optimal gap codes
A rayon-parallelized utility to determine which encoding is more space efficient for a given
precision, number of bits, hash size and hasher.

## How to run
As usual, just run:

```bash
RUSTFLAGS='-C target-cpu=native' cargo run --release
```

## Results
The results will be stored in the [`optimal_gap_codes.csv`]() file. Here is a preview of 
a few lines:

| precision | bit_size | hash_size | hasher    | composite_hash | code        | space_usage   | uncompressed_space_usage | rate                |
|-----------|----------|-----------|-----------|----------------|-------------|---------------|--------------------------|---------------------|
| 18        | 5        | 24        | WyHash    | SwitchHash     | Rice(8)     | 106718182096  | 262137600024              | 0.4071074965446751   |
| 18        | 5        | 24        | XxHash64  | SwitchHash     | Rice(8)     | 106718194332  | 262137600024              | 0.4071075432224504   |
| 18        | 5        | 24        | AHasher   | SwitchHash     | Rice(8)     | 106718247516  | 262137600024              | 0.40710774610826306  |
| 18        | 4        | 24        | XxHash64  | SwitchHash     | Rice(8)     | 87879427304   | 209707200024              | 0.41905774953813035  |
| 18        | 4        | 24        | WyHash    | SwitchHash     | Rice(8)     | 87879465666   | 209707200024              | 0.4190579324693793   |
| 18        | 4        | 24        | AHasher   | SwitchHash     | Rice(8)     | 87879521262   | 209707200024              | 0.41905819758187896  |
| 17        | 6        | 24        | WyHash    | SwitchHash     | Rice(8)     | 69099457554   | 157281600024              | 0.43933592704713037  |
| 17        | 6        | 24        | XxHash64  | SwitchHash     | Rice(8)     | 69099458840   | 157281600024              | 0.4393359352235477   |
| 17        | 6        | 24        | AHasher   | SwitchHash     | Rice(8)     | 69099496406   | 157281600024              | 0.4393361740690324   |
| 17        | 5        | 24        | XxHash64  | SwitchHash     | Rice(9)     | 58812889480   | 131064000024              | 0.4487341258410424   |
| 17        | 5        | 24        | WyHash    | SwitchHash     | Rice(9)     | 58812910934   | 131064000024              | 0.44873428953206357  |
| 17        | 5        | 24        | AHasher   | SwitchHash     | Rice(9)     | 58812936536   | 131064000024              | 0.4487344848717449   |
