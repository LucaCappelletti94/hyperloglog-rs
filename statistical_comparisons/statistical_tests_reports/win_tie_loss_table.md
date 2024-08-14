| task        | approach                 |   win |   tie |   loss |     error |
|:------------|:-------------------------|------:|------:|-------:|----------:|
| cardinality | HybridBeta + WyHash      |   137 |    65 |     38 | 0.0465152 |
| cardinality | Hybrid++ + WyHash        |   117 |    69 |     54 | 0.0465736 |
| cardinality | HLL8 + WyHash            |   109 |    51 |     80 | 0.0466342 |
| cardinality | HLL6 + WyHash            |   109 |    51 |     80 | 0.0466342 |
| cardinality | Cardinality Estimator    |   174 |    39 |     27 | 0.0466857 |
| cardinality | Tabac's HLL++            |   143 |    22 |     75 | 0.0469761 |
| cardinality | Rust-HLL                 |   112 |    32 |     96 | 0.0470738 |
| cardinality | HybridBeta + Xxhasher    |   130 |    67 |     43 | 0.0471461 |
| cardinality | Hybrid++ + Xxhasher      |   104 |    69 |     67 | 0.0472029 |
| cardinality | Streaming Algorithms     |   142 |    34 |     64 | 0.047216  |
| cardinality | Beta8 + WyHash           |    43 |    39 |    158 | 0.0472627 |
| cardinality | Beta6 + WyHash           |    43 |    39 |    158 | 0.0472627 |
| cardinality | HLL8 + Xxhasher          |   100 |    49 |     91 | 0.0472635 |
| cardinality | HLL6 + Xxhasher          |   100 |    49 |     91 | 0.0472635 |
| cardinality | Tabac's HLL              |    56 |     7 |    177 | 0.0477704 |
| cardinality | Beta8 + Xxhasher         |    22 |    36 |    182 | 0.0478932 |
| cardinality | Beta6 + Xxhasher         |    22 |    36 |    182 | 0.0478932 |
| union       | MLE++ + Xxhasher         |   242 |    23 |     47 | 0.0421415 |
| union       | MLEBeta + Xxhasher       |   248 |    23 |     41 | 0.0421425 |
| union       | HybridMLEBeta + Xxhasher |   202 |    34 |     76 | 0.043106  |
| union       | HybridMLE++ + Xxhasher   |   199 |    34 |     79 | 0.0431096 |
| union       | HybridBeta + Xxhasher    |   180 |    38 |     94 | 0.0451333 |
| union       | Hybrid++ + Xxhasher      |   169 |    41 |    102 | 0.0451603 |
| union       | HLL8 + Xxhasher          |   171 |    32 |    109 | 0.0452603 |
| union       | HLL6 + Xxhasher          |   171 |    32 |    109 | 0.0452603 |
| union       | Beta8 + Xxhasher         |   130 |    31 |    151 | 0.0457177 |
| union       | Beta6 + Xxhasher         |   130 |    31 |    151 | 0.0457177 |
| union       | MLE++ + WyHash           |   194 |    25 |     93 | 0.0465289 |
| union       | MLEBeta + WyHash         |   190 |    25 |     97 | 0.0465313 |
| union       | HybridMLEBeta + WyHash   |   162 |    27 |    123 | 0.0475344 |
| union       | HybridMLE++ + WyHash     |   159 |    27 |    126 | 0.0475345 |
| union       | Cardinality Estimator    |   194 |     2 |    116 | 0.0476969 |
| union       | HybridBeta + WyHash      |   113 |    33 |    166 | 0.0507617 |
| union       | Hybrid++ + WyHash        |   104 |    33 |    175 | 0.0507924 |
| union       | HLL8 + WyHash            |    95 |    35 |    182 | 0.0508966 |
| union       | HLL6 + WyHash            |    95 |    35 |    182 | 0.0508966 |
| union       | Beta8 + WyHash           |    56 |    28 |    228 | 0.0513459 |
| union       | Beta6 + WyHash           |    56 |    28 |    228 | 0.0513459 |
| union       | Rust-HLL                 |   161 |    11 |    140 | 0.0520943 |
| union       | Streaming Algorithms     |   149 |     6 |    157 | 0.0550543 |
| union       | Tabac's HLL++            |     4 |     3 |    305 | 0.257252  |
| union       | Tabac's HLL              |     6 |     3 |    303 | 0.257709  |