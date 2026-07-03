# `ZERO_BITS[P-4][B-4]`: low-mantissa bit width reserved for the packed zero-register count

One entry per `(P, B)` cell. `ZERO_BITS[p][b]` is the number of low mantissa bits of the
`HyperLogLog::harmonic_sum` f64 word that hold the zero-register count in the dense
bias-corrected band; the remaining `52 - ZERO_BITS[p][b]` mantissa bits carry the
harmonic sum. In the raw regime and in pre-dense representations the word carries no
packing; in dense zeros mode the word is the full NaN-boxed zero count, unchanged.

Source: `examples/zero_count_probe.rs`, run at 2026-07-03 on `hll-union-merger`. Probe
output preserved at `/tmp/zero_probe_out.txt` (P18 B6 line dropped by a 3600s timeout;
value taken from the analytical prediction, which matches every other cell within 1 count).

## Derivation

In harmonic mode the linear-count estimate is above `HYPERLOGLOG_LINEAR_COUNT_THRESHOLD[p][b]`
by construction (`finalize_dense_representation`), and inserts monotonically decrement the
zero count. So the worst-case zero count in the band is `zeros_max = m * exp(-threshold / m)`,
and `ZERO_BITS[p][b] = max(ceil(log2(zeros_max + 1)) + 2, 4)`.

The `+2` safety margin covers off-by-one at the crossover and future threshold-table changes;
the `4`-bit floor uniformizes packing at small P where `zeros_max` collapses to 0.

Perturbation on `H` from stealing 16 low bits at magnitude `~0.1 * m` (worst case P15/P16):
relative error `2^(16 - 52) = 2^-36 ~ 1.5e-11`, six orders below the register noise floor.

## Full table

| P  | B | threshold | max\_zeros (measured) | max\_zeros (theory) | raw bits | `ZERO_BITS` |
|---:|--:|----------:|----------------------:|--------------------:|---------:|------------:|
|  4 | 4 |        45 |                     0 |                   1 |        1 |           4 |
|  4 | 5 |        45 |                     0 |                   1 |        1 |           4 |
|  4 | 6 |        45 |                     0 |                   1 |        1 |           4 |
|  5 | 4 |        54 |                     5 |                   6 |        3 |           5 |
|  5 | 5 |       104 |                     1 |                   1 |        1 |           4 |
|  5 | 6 |       104 |                     1 |                   1 |        1 |           4 |
|  6 | 4 |       108 |                    11 |                  12 |        4 |           6 |
|  6 | 5 |       128 |                     8 |                   9 |        4 |           6 |
|  6 | 6 |       139 |                     7 |                   7 |        3 |           5 |
|  7 | 4 |       202 |                    26 |                  26 |        5 |           7 |
|  7 | 5 |       257 |                    17 |                  17 |        5 |           7 |
|  7 | 6 |       299 |                    12 |                  12 |        4 |           6 |
|  8 | 4 |       367 |                    61 |                  61 |        6 |           8 |
|  8 | 5 |       490 |                    37 |                  38 |        6 |           8 |
|  8 | 6 |      1272 |                     1 |                   2 |        2 |           4 |
|  9 | 4 |       664 |                   139 |                 140 |        8 |          10 |
|  9 | 5 |       902 |                    87 |                  88 |        7 |           9 |
|  9 | 6 |      2880 |                     1 |                   2 |        2 |           4 |
| 10 | 4 |      1332 |                   278 |                 279 |        9 |          11 |
| 10 | 5 |      1804 |                   175 |                 176 |        8 |          10 |
| 10 | 6 |      2320 |                   106 |                 106 |        7 |           9 |
| 11 | 4 |      2632 |                   566 |                 566 |       10 |          12 |
| 11 | 5 |      3576 |                   357 |                 357 |        9 |          11 |
| 11 | 6 |      4592 |                   217 |                 218 |        8 |          10 |
| 12 | 4 |      5280 |                  1128 |                1129 |       11 |          13 |
| 12 | 5 |      7088 |                   725 |                 726 |       10 |          12 |
| 12 | 6 |      9184 |                   435 |                 435 |        9 |          11 |
| 13 | 4 |     10464 |                  2283 |                2284 |       12 |          14 |
| 13 | 5 |     14208 |                  1445 |                1446 |       11 |          13 |
| 13 | 6 |     18240 |                   883 |                 884 |       10 |          12 |
| 14 | 4 |     23744 |                  3846 |                3846 |       12 |          14 |
| 14 | 5 |     29184 |                  2759 |                2760 |       12 |          14 |
| 14 | 6 |     37120 |                  1700 |                1700 |       11 |          13 |
| 15 | 4 |     36480 |                 10763 |               10764 |       14 |          16 |
| 15 | 5 |     52352 |                  6631 |                6631 |       13 |          15 |
| 15 | 6 |     70912 |                  3763 |                3764 |       12 |          14 |
| 16 | 4 |    133120 |                  8596 |                8596 |       14 |          16 |
| 16 | 5 |    145408 |                  7126 |                7127 |       13 |          15 |
| 16 | 6 |    140288 |                  7705 |                7706 |       13 |          15 |
| 17 | 4 |    575488 |                  1624 |                1624 |       11 |          13 |
| 17 | 5 |    589824 |                  1456 |                1456 |       11 |          13 |
| 17 | 6 |    589824 |                  1456 |                1456 |       11 |          13 |
| 18 | 4 |   1376256 |                  1375 |                1376 |       11 |          13 |
| 18 | 5 |   1372160 |                  1397 |                1397 |       11 |          13 |
| 18 | 6 |   1372160 |                (pred) |                1397 |       11 |          13 |

## The frozen const, ready to paste into `src/hyperloglog.rs`

```rust
/// Low-mantissa bit width reserved for the packed zero-register count in the
/// dense bias-corrected band, indexed `[P - 4][B - 4]`. See
/// `docs/zero_bits_table.md` for the derivation and the probe run that fixed it.
const ZERO_BITS: [[u8; 3]; 15] = [
    [ 4u8,  4u8,  4u8], // P4   max zeros ~      1 /      1 /      1
    [ 5u8,  4u8,  4u8], // P5   max zeros ~      6 /      1 /      1
    [ 6u8,  6u8,  5u8], // P6   max zeros ~     12 /      9 /      7
    [ 7u8,  7u8,  6u8], // P7   max zeros ~     26 /     17 /     12
    [ 8u8,  8u8,  4u8], // P8   max zeros ~     61 /     38 /      2
    [10u8,  9u8,  4u8], // P9   max zeros ~    140 /     88 /      2
    [11u8, 10u8,  9u8], // P10  max zeros ~    279 /    176 /    106
    [12u8, 11u8, 10u8], // P11  max zeros ~    566 /    357 /    218
    [13u8, 12u8, 11u8], // P12  max zeros ~   1129 /    726 /    435
    [14u8, 13u8, 12u8], // P13  max zeros ~   2284 /   1446 /    884
    [14u8, 14u8, 13u8], // P14  max zeros ~   3846 /   2760 /   1700
    [16u8, 15u8, 14u8], // P15  max zeros ~  10764 /   6631 /   3764
    [16u8, 15u8, 15u8], // P16  max zeros ~   8596 /   7127 /   7706
    [13u8, 13u8, 13u8], // P17  max zeros ~   1624 /   1456 /   1456
    [13u8, 13u8, 13u8], // P18  max zeros ~   1376 /   1397 /   1397
];
```

