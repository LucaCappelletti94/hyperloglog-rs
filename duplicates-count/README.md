# Duplicate Count
A rayon-parallelized utility to determine how many duplicated subquent hash may arise after a 
degradation event of the HashList data structure.

## How to run
As usual, just run:

```bash
RUSTFLAGS='-C target-cpu=native' cargo run --release
```

## Results
Here is a preview of the results, which are stored in the [`duplicates.csv`](https://github.com/LucaCappelletti94/hyperloglog-rs/blob/main/duplicates-count/duplicates.csv) file:

| precision | bit_size | starting_hash_size | downgraded_hash_size | hasher    | composite_hash | average_number_of_duplicates | absolute_average_number_of_duplicates | total_number_of_hashes |
|-----------|----------|--------------------|----------------------|-----------|----------------|-----------------------------|---------------------------------------|------------------------|
| 8         | 5        | 24                 | 16                   | XxHash64  | CurrentHash    | 0.0041154647099930115        | 0.2222                                | 54                     |
| 8         | 5        | 24                 | 16                   | AHasher   | CurrentHash    | 0.0041606848357791755        | 0.22464                               | 54                     |
| 8         | 5        | 24                 | 16                   | WyHash    | CurrentHash    | 0.004168518518518518         | 0.22506                               | 54                     |
| 7         | 6        | 24                 | 16                   | AHasher   | CurrentHash    | 0.0048439393939393935        | 0.1598                                | 33                     |
| 7         | 6        | 24                 | 16                   | WyHash    | CurrentHash    | 0.004887121212121212         | 0.16122                               | 33                     |
| 7         | 6        | 24                 | 16                   | XxHash64  | CurrentHash    | 0.004931079545454546         | 0.16268                               | 33                     |
| 10        | 4        | 24                 | 16                   | XxHash64  | CurrentHash    | 0.006753382868937048         | 1.15474                               | 171                    |
| 10        | 4        | 24                 | 16                   | AHasher   | CurrentHash    | 0.0067765730994152045        | 1.1587                                | 171                    |
| 10        | 4        | 24                 | 16                   | WyHash    | CurrentHash    | 0.006800567595459235         | 1.1628                                | 171                    |
| 9         | 5        | 24                 | 16                   | WyHash    | CurrentHash    | 0.008361812731440661         | 0.89458                               | 107                    |
| 9         | 5        | 24                 | 16                   | AHasher   | CurrentHash    | 0.008376907071063304         | 0.8962                                | 107                    |
| 9         | 5        | 24                 | 16                   | XxHash64  | CurrentHash    | 0.008389037206841825         | 0.89748                               | 107                    |
| 8         | 6        | 24                 | 16                   | XxHash64  | CurrentHash    | 0.009956538461538461         | 0.64696                               | 65                     |
| 8         | 6        | 24                 | 16                   | WyHash    | CurrentHash    | 0.009992995192307692         | 0.64934                               | 65                     |
| 8         | 6        | 24                 | 16                   | AHasher   | CurrentHash    | 0.010015961538461538         | 0.65082                               | 65                     |
| 4         | 4        | 16                 | 8                    | WyHash    | CurrentHash    | 0.024658000000000003         | 0.1214                                | 5                      |
| 4         | 4        | 16                 | 8                    | WyHash    | SwitchHash     | 0.02468                     | 0.1215                                | 5                      |
| 4         | 4        | 16                 | 8                    | XxHash64  | CurrentHash    | 0.024795                    | 0.122                                 | 5                      |
| 4         | 4        | 16                 | 8                    | XxHash64  | SwitchHash     | 0.024833999999999995         | 0.1222                                | 5                      |
| 4         | 4        | 16                 | 8                    | AHasher   | CurrentHash    | 0.025378                    | 0.1249                                | 4                      |
| 4         | 4        | 16                 | 8                    | AHasher   | SwitchHash     | 0.02544                     | 0.12518                               | 4                      |
