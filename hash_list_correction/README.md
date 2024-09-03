# Hash List correction
The Hash List low-cardinality correction approach for HyperLogLog counters is itself subject to biases, primarily derived by the so-called birthday paradox, i.e. the probability of two elements in a set of size n to have the same hash value is higher than one might expect. This is a well-known issue in computer science and cryptography, and it is the reason why hash functions are designed to be collision-resistant. Fortunately, while theoretically hard to estimate because of the several techniques we employ, we can empirically measure the bias introduced by the Hash List correction approach.

Once the error is measured, we can trivially subtract it to the cardinality estimate and obtain a much nicer result. This is the purpose of the `hash_list_correction` module, which provides a simple interface to measure the bias and correct the cardinality estimate.

## Usage
As any rust script, just use:

```bash
RUSTFLAGS='-C target-cpu=native' cargo run --release
```

## Switch Hash Results
The results regarding the switch hash are as follows:

| Precision | Bits | Rate of Improvement |
|-----------|------|---------------------|
| 4         | 4    | 9.84                |
| 4         | 5    | 1093.55             |
| 4         | 6    | 57.16               |
| 5         | 4    | 638.58              |
| 5         | 5    | 62.63               |
| 5         | 6    | 62.05               |
| 6         | 4    | 192.30              |
| 6         | 5    | 160.66              |
| 6         | 6    | 213.25              |
| 7         | 4    | 135.74              |
| 7         | 5    | 533.09              |
| 7         | 6    | 730.71              |
| 8         | 4    | 43.42               |
| 8         | 5    | 36.66               |
| 8         | 6    | 33.75               |
| 9         | 4    | 28.21               |
| 9         | 5    | 26.99               |
| 9         | 6    | 25.76               |
| 10        | 4    | 30.10               |
| 10        | 5    | 21.33               |
| 10        | 6    | 6.97                |
| 11        | 4    | 38.36               |
| 11        | 5    | 9.01                |
| 11        | 6    | 510.89              |
| 12        | 4    | 13.82               |
| 12        | 5    | 859.49              |
| 12        | 6    | 893.94              |
| 13        | 4    | 656.76              |
| 13        | 5    | 759.58              |
| 13        | 6    | 1294.57             |
| 14        | 4    | 29.52               |
| 14        | 5    | 27.40               |
| 14        | 6    | 27.33               |
| 15        | 4    | 17.90               |
| 15        | 5    | 25.38               |
| 15        | 6    | 45.90               |
| 16        | 4    | 3.56                |
| 16        | 5    | 28.21               |
| 16        | 6    | 9.40                |
| 17        | 4    | 29.55               |
| 17        | 5    | 31.13               |
| 17        | 6    | 3.93                |
| 18        | 4    | 1.14                |
| 18        | 5    | 62.59               |
| 18        | 6    | 83.04               |
