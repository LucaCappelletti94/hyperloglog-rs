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

| precision | bits | maximal_mean_relative_error | peak_estimated_cardinality | bias | error_reduction |
|-----------|------|-----------------------------|-----------------------------|------|-----------------|
| 4         | 4    | 0.0691152                   | 7.45                        | 1.00 | 56.22           |
| 4         | 5    | 0.0001191                   | 8.00                        | 1.00 | 156.17          |
| 4         | 6    | 0.0001077                   | 8.00                        | 1.00 | 109.75          |
| 5         | 4    | 0.0001238                   | 8.00                        | 1.00 | 105.03          |
| 5         | 5    | 0.0001798                   | 12.00                       | 1.00 | 57.33           |
| 5         | 6    | 0.0001709                   | 12.00                       | 1.00 | 70.69           |
| 6         | 4    | 0.0002604                   | 16.00                       | 1.00 | 191.23          |
| 6         | 5    | 0.0002986                   | 19.99                       | 1.00 | 162.92          |
| 6         | 6    | 0.0003535                   | 23.99                       | 1.00 | 221.90          |
| 7         | 4    | 0.0004990                   | 31.98                       | 1.00 | 826.10          |
| 7         | 5    | 0.0005987                   | 39.98                       | 1.00 | 751.54          |
| 7         | 6    | 0.0007155                   | 47.97                       | 1.00 | 668.20          |
| 8         | 4    | 0.0005582                   | 63.96                       | 41.00 | 26.07           |
| 8         | 5    | 0.0006774                   | 79.95                       | 51.00 | 24.80           |
| 8         | 6    | 0.0008063                   | 95.92                       | 62.00 | 26.04           |
| 9         | 4    | 0.0010797                   | 127.86                      | 83.00 | 25.15           |
| 9         | 5    | 0.0013529                   | 159.78                      | 103.00 | 25.56           |
| 9         | 6    | 0.0016084                   | 191.69                      | 125.00 | 25.72           |
| 10        | 4    | 0.0021341                   | 255.45                      | 165.00 | 26.05           |
| 10        | 5    | 0.0026572                   | 319.15                      | 207.00 | 24.27           |
| 10        | 6    | 0.0349638                   | 379.62                      | 251.00 | 17.03           |
| 11        | 4    | 0.0041498                   | 509.88                      | 331.99 | 21.21           |
| 11        | 5    | 0.0293490                   | 637.79                      | 414.99 | 17.81           |
| 11        | 6    | 0.0000331                   | 511.98                      | 1.00 | 207.34          |
| 12        | 4    | 0.0235120                   | 1021.98                     | 664.97 | 20.97           |
| 12        | 5    | 0.0000586                   | 852.95                      | 1.00 | 203.13          |
| 12        | 6    | 0.0000633                   | 1023.94                     | 1.00 | 283.56          |
| 13        | 4    | 0.0000954                   | 1364.87                     | 1.00 | 292.92          |
| 13        | 5    | 0.0001082                   | 1705.82                     | 1.00 | 302.12          |
| 13        | 6    | 0.0001232                   | 2047.75                     | 1.00 | 352.58          |
| 14        | 4    | 0.0000813                   | 2729.78                     | 2017.00 | 22.68           |
| 14        | 5    | 0.0000917                   | 3412.69                     | 2523.00 | 22.42           |
| 14        | 6    | 0.0001078                   | 4095.56                     | 3028.00 | 22.78           |
| 15        | 4    | 0.0001507                   | 5460.18                     | 4034.00 | 23.08           |
| 15        | 5    | 0.0001801                   | 6824.77                     | 5040.99 | 22.02           |
| 15        | 6    | 0.0002143                   | 8190.24                     | 6049.99 | 21.92           |
| 16        | 4    | 0.0002903                   | 10918.83                    | 8058.98 | 22.23           |
| 16        | 5    | 0.0003558                   | 13649.14                    | 10082.97 | 21.80           |
| 16        | 6    | 0.0004274                   | 16380.00                    | 12106.96 | 21.89           |
| 17        | 4    | 0.0005662                   | 21838.63                    | 16133.93 | 21.76           |
| 17        | 5    | 0.0007094                   | 27306.61                    | 20172.88 | 21.90           |
| 17        | 6    | 0.0008523                   | 32772.45                    | 24199.83 | 22.47           |
| 18        | 4    | 0.0011257                   | 43711.11                    | 32258.70 | 22.13           |
| 18        | 5    | 0.0014110                   | 54661.67                    | 40339.54 | 22.14           |
| 18        | 6    | 0.0198708                   | 66823.58                    | 48436.36 | 35.60           |