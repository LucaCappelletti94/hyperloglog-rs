# Measure Gaps
A rayon-parallelized utility to measure sorted hash gaps.

## How to run
As usual, just run:

```bash
RUSTFLAGS='-C target-cpu=native' cargo run --release
```

## Results
The results will be stored in the `reports` directory in this directory, as a gzipped CSV file.
The header of the CSV file is as follows, and is sorted by the `gap` column:

```
gap,count
3,2323
4,123
5,123
```

