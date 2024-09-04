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

| precision | bit_size | hash_size | vbyte | code      | rate   | mean_compressed_size | number_of_hashes | number_of_hashes_with_code | extra_hashes |
|-----------|----------|-----------|-------|-----------|--------|----------------------|------------------|----------------------------|--------------|
| 17        | 6        | 29        | true  | Rice(10)  | 0.5322 | 17.0292              | 24576            | 46260                      | 21684        |
| 17        | 6        | 30        | false | Rice(10)  | 0.4797 | 15.3519              | 24576            | 52428                      | 27852        |
| 17        | 6        | 30        | true  | Rice(11)  | 0.5586 | 17.8751              | 24576            | 46260                      | 21684        |
| 17        | 6        | 31        | false | Rice(11)  | 0.5114 | 16.3635              | 24576            | 49152                      | 24576        |
| 17        | 6        | 31        | true  | Rice(11)  | 0.6000 | 19.2003              | 24576            | 41391                      | 16815        |
| 17        | 6        | 32        | false | Rice(12)  | 0.5430 | 17.3753              | 24576            | 46260                      | 21684        |
| 17        | 6        | 32        | true  | Rice(12)  | 0.6418 | 20.5361              | 24576            | 39321                      | 14745        |
| 18        | 4        | 22        | false | Rice(2)   | 0.2810 | 6.7450               | 43690            | 174762                     | 131072       |
| 18        | 4        | 22        | true  | Rice(2)   | 0.3958 | 9.4984               | 43690            | 116508                     | 72818        |
| 18        | 4        | 23        | false | Rice(3)   | 0.3265 | 7.8363               | 43690            | 149796                     | 106106       |
| 18        | 4        | 23        | true  | Rice(3)   | 0.4461 | 10.7063              | 43690            | 104857                     | 61167        |
| 18        | 4        | 24        | false | Rice(4)   | 0.3667 | 8.8019               | 43690            | 131072                     | 87382        |
| 18        | 4        | 24        | true  | Rice(4)   | 0.4960 | 11.9048              | 43690            | 95325                      | 51635        |
| 18        | 4        | 25        | false | Rice(5)   | 0.3065 | 9.8085               | 32768            | 116508                     | 83740        |
| 18        | 4        | 25        | true  | Rice(4)   | 0.4208 | 13.4656              | 32768            | 80659                      | 47891        |
| 18        | 4        | 26        | false | Rice(6)   | 0.3386 | 10.8348              | 32768            | 104857                     | 72089        |
| 18        | 4        | 26        | true  | Rice(5)   | 0.4734 | 15.1480              | 32768            | 69905                      | 37137        |
| 18        | 4        | 27        | false | Rice(7)   | 0.3710 | 11.8712              | 32768            | 95325                      | 62557        |
| 18        | 4        | 27        | true  | Rice(8)   | 0.5072 | 16.2317              | 32768            | 65536                      | 32768        |
