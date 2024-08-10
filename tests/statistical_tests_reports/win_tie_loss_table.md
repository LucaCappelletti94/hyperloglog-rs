| task        | approach              |   win |   tie |   loss |     error |
|:------------|:----------------------|------:|------:|-------:|----------:|
| cardinality | Cardinality Estimator |     5 |    94 |      5 | 0.0539113 |
| cardinality | Tabac's HLL           |     0 |    78 |     26 | 0.0540118 |
| cardinality | Beta6 + WyHash        |    14 |    86 |      4 | 0.0540843 |
| cardinality | HLL6 + WyHash         |    11 |    84 |      9 | 0.0541014 |
| cardinality | Streaming Algorithms  |     7 |    96 |      1 | 0.0541398 |
| cardinality | Tabac's HLL++         |     5 |    96 |      3 | 0.0542468 |
| cardinality | Beta6 + Xxhasher      |     7 |    93 |      4 | 0.0542568 |
| cardinality | HLL6 + Xxhasher       |     6 |    90 |      8 | 0.0542729 |
| cardinality | Rust-HLL              |     6 |    97 |      1 | 0.0545624 |
| union       | MLE2 + Xxhasher       |    76 |     1 |      1 | 0.045496  |
| union       | Cardinality Estimator |    41 |    20 |     17 | 0.0507382 |
| union       | HLL6 + Xxhasher       |    41 |    15 |     22 | 0.0510594 |
| union       | Streaming Algorithms  |    31 |    18 |     29 | 0.0527505 |
| union       | Rust-HLL              |    32 |    20 |     26 | 0.0541614 |
| union       | Tabac's HLL++         |     1 |    14 |     63 | 0.0731294 |
| union       | Tabac's HLL           |     0 |    14 |     64 | 0.0733075 |