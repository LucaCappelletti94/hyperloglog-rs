| task        | approach              |   win |   tie |   loss |     error |
|:------------|:----------------------|------:|------:|-------:|----------:|
| cardinality | HLL6 + WyHash         |     9 |    90 |      5 | 0.0535833 |
| cardinality | Beta6 + WyHash        |    10 |    90 |      4 | 0.0535908 |
| cardinality | Tabac's HLL           |     2 |    82 |     20 | 0.0537384 |
| cardinality | Cardinality Estimator |     9 |    89 |      6 | 0.0539064 |
| cardinality | Streaming Algorithms  |     4 |    98 |      2 | 0.0542157 |
| cardinality | HLL6 + Xxhasher       |     8 |    90 |      6 | 0.0542528 |
| cardinality | Beta6 + Xxhasher      |     6 |    91 |      7 | 0.0542666 |
| cardinality | Rust-HLL              |     4 |    95 |      5 | 0.0546064 |
| cardinality | Tabac's HLL++         |    10 |    87 |      7 | 0.0549599 |
| union       | MLE2 + Xxhasher       |   104 |     6 |     20 | 0.0411672 |
| union       | HLL6 + Xxhasher       |    75 |     9 |     46 | 0.0440771 |
| union       | MLE2 + WyHash         |    82 |     9 |     39 | 0.0442799 |
| union       | HLL6 + WyHash         |    52 |     7 |     71 | 0.0486364 |
| union       | Cardinality Estimator |    76 |     8 |     46 | 0.0511134 |
| union       | Rust-HLL              |    71 |    11 |     48 | 0.0524634 |
| union       | Streaming Algorithms  |    65 |    10 |     55 | 0.0566335 |
| union       | MLE2 + DefaultHasher  |    84 |     4 |     42 | 0.0606102 |
| union       | HLL6 + DefaultHasher  |    55 |    10 |     65 | 0.0645425 |
| union       | Tabac's HLL++         |     1 |    13 |    116 | 0.229244  |
| union       | Tabac's HLL           |     0 |    13 |    117 | 0.229465  |