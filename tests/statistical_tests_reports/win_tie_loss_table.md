| task        | approach             |   win |   tie |   loss |
|:------------|:---------------------|------:|------:|-------:|
| cardinality | HLL6 + Xxhasher      |    77 |    28 |     38 |
| cardinality | HLL5 + Xxhasher      |    77 |    28 |     38 |
| cardinality | Streaming Algorithms |    63 |    29 |     51 |
| cardinality | HLL4 + Xxhasher      |    62 |    26 |     55 |
| cardinality | HLL4 + Siphasher13   |    60 |    32 |     51 |
| cardinality | Tabac's HLL++        |    51 |    38 |     54 |
| cardinality | HLL6 + Siphasher24   |    51 |    37 |     55 |
| cardinality | HLL5 + Siphasher24   |    51 |    37 |     55 |
| cardinality | HLL6 + Siphasher13   |    47 |    36 |     60 |
| cardinality | HLL5 + Siphasher13   |    47 |    36 |     60 |
| cardinality | HLL4 + Siphasher24   |    43 |    25 |     75 |
| cardinality | Tabac's HLL          |    35 |    36 |     72 |
| union       | MLE3 + Xxhasher      |   175 |    20 |     26 |
| union       | MLE2 + Xxhasher      |   175 |    20 |     26 |
| union       | MLE3 + Siphasher13   |   152 |    13 |     56 |
| union       | MLE2 + Siphasher13   |   138 |    18 |     65 |
| union       | MLE3 + Siphasher24   |   130 |    16 |     75 |
| union       | HLL6 + Xxhasher      |   121 |    24 |     76 |
| union       | HLL5 + Xxhasher      |   121 |    24 |     76 |
| union       | MLE2 + Siphasher24   |   120 |    14 |     87 |
| union       | Streaming Algorithms |   113 |    19 |     89 |
| union       | HLL4 + Siphasher13   |   100 |    12 |    109 |
| union       | HLL4 + Xxhasher      |    99 |    15 |    107 |
| union       | HLL6 + Siphasher13   |    85 |    23 |    113 |
| union       | HLL5 + Siphasher13   |    85 |    23 |    113 |
| union       | HLL6 + Siphasher24   |    71 |    29 |    121 |
| union       | HLL5 + Siphasher24   |    71 |    29 |    121 |
| union       | HLL4 + Siphasher24   |    58 |    15 |    148 |
| union       | Tabac's HLL++        |     4 |    12 |    205 |
| union       | Tabac's HLL          |     2 |    12 |    207 |