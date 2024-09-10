//! Optimal codes for the gap between subsequent hashes in the Listhash variant of HyperLogLog.
/// The optimal Rice code coefficients for the different precisions and bit sizes, when using hash-packing.
pub(super) const OPTIMAL_RICE_COEFFICIENTS: [[&[(u8, u8, u8)]; 3]; 15] = [
    [&[(8u8, 0u8, 1u8)], &[(9u8, 0u8, 1u8)], &[(10u8, 0u8, 0u8)]],
    [&[(9u8, 0u8, 0u8)], &[(10u8, 0u8, 0u8)], &[(11u8, 0u8, 0u8)]],
    [&[(10u8, 0u8, 0u8)], &[(11u8, 0u8, 0u8)], &[(12u8, 0u8, 0u8), (13u8, 7u8, 0u8)]],
    [
        &[(11u8, 0u8, 0u8), (12u8, 6u8, 0u8), (13u8, 7u8, 0u8), (14u8, 8u8, 0u8)],
        &[(12u8, 0u8, 0u8), (13u8, 7u8, 0u8), (14u8, 8u8, 0u8)],
        &[(13u8, 0u8, 0u8), (14u8, 7u8, 0u8)],
    ],
    [
        &[
            (12u8, 0u8, 0u8),
            (13u8, 6u8, 0u8),
            (14u8, 7u8, 0u8),
            (15u8, 8u8, 0u8),
            (16u8, 9u8, 0u8),
            (17u8, 10u8, 0u8),
            (18u8, 11u8, 0u8),
            (19u8, 12u8, 0u8),
            (20u8, 14u8, 0u8),
            (21u8, 15u8, 0u8),
        ],
        &[
            (13u8, 0u8, 0u8),
            (14u8, 7u8, 0u8),
            (15u8, 8u8, 0u8),
            (16u8, 9u8, 0u8),
            (17u8, 10u8, 0u8),
            (18u8, 11u8, 0u8),
            (19u8, 12u8, 0u8),
            (20u8, 13u8, 0u8),
            (21u8, 14u8, 0u8),
        ],
        &[
            (14u8, 0u8, 0u8),
            (15u8, 7u8, 0u8),
            (16u8, 9u8, 0u8),
            (17u8, 10u8, 0u8),
            (18u8, 11u8, 0u8),
            (19u8, 12u8, 0u8),
            (20u8, 13u8, 0u8),
            (21u8, 14u8, 0u8),
        ],
    ],
    [
        &[
            (13u8, 0u8, 0u8),
            (14u8, 6u8, 0u8),
            (15u8, 7u8, 0u8),
            (16u8, 8u8, 0u8),
            (17u8, 9u8, 0u8),
            (18u8, 10u8, 0u8),
            (19u8, 11u8, 0u8),
            (20u8, 12u8, 0u8),
            (21u8, 14u8, 0u8),
            (22u8, 15u8, 0u8),
        ],
        &[
            (14u8, 0u8, 0u8),
            (15u8, 7u8, 0u8),
            (16u8, 8u8, 0u8),
            (17u8, 9u8, 0u8),
            (18u8, 10u8, 0u8),
            (19u8, 11u8, 0u8),
            (20u8, 12u8, 0u8),
            (21u8, 13u8, 0u8),
            (22u8, 14u8, 0u8),
        ],
        &[
            (15u8, 0u8, 0u8),
            (16u8, 7u8, 0u8),
            (17u8, 9u8, 0u8),
            (18u8, 10u8, 0u8),
            (19u8, 11u8, 0u8),
            (20u8, 12u8, 0u8),
            (21u8, 13u8, 0u8),
            (22u8, 14u8, 0u8),
        ],
    ],
    [
        &[
            (14u8, 0u8, 0u8),
            (15u8, 6u8, 0u8),
            (16u8, 7u8, 0u8),
            (17u8, 8u8, 0u8),
            (18u8, 9u8, 0u8),
            (19u8, 10u8, 0u8),
            (20u8, 11u8, 0u8),
            (21u8, 12u8, 0u8),
            (22u8, 14u8, 0u8),
            (23u8, 15u8, 0u8),
        ],
        &[
            (15u8, 0u8, 0u8),
            (16u8, 7u8, 0u8),
            (17u8, 8u8, 0u8),
            (18u8, 9u8, 0u8),
            (19u8, 10u8, 0u8),
            (20u8, 11u8, 0u8),
            (21u8, 12u8, 0u8),
            (22u8, 13u8, 0u8),
            (23u8, 14u8, 0u8),
        ],
        &[
            (16u8, 0u8, 0u8),
            (17u8, 7u8, 0u8),
            (18u8, 9u8, 0u8),
            (19u8, 10u8, 0u8),
            (20u8, 11u8, 0u8),
            (21u8, 12u8, 0u8),
            (22u8, 13u8, 0u8),
            (23u8, 14u8, 0u8),
        ],
    ],
    [
        &[
            (15u8, 0u8, 0u8),
            (16u8, 6u8, 0u8),
            (17u8, 7u8, 0u8),
            (18u8, 8u8, 0u8),
            (19u8, 9u8, 0u8),
            (20u8, 10u8, 0u8),
            (21u8, 11u8, 0u8),
            (22u8, 13u8, 0u8),
            (23u8, 14u8, 0u8),
            (24u8, 15u8, 0u8),
        ],
        &[
            (16u8, 0u8, 0u8),
            (17u8, 7u8, 0u8),
            (18u8, 8u8, 0u8),
            (19u8, 9u8, 0u8),
            (20u8, 10u8, 0u8),
            (21u8, 11u8, 0u8),
            (22u8, 12u8, 0u8),
            (23u8, 13u8, 0u8),
            (24u8, 14u8, 0u8),
        ],
        &[
            (17u8, 0u8, 0u8),
            (18u8, 7u8, 0u8),
            (19u8, 9u8, 0u8),
            (20u8, 10u8, 0u8),
            (21u8, 11u8, 0u8),
            (22u8, 12u8, 0u8),
            (23u8, 13u8, 0u8),
            (24u8, 14u8, 0u8),
        ],
    ],
    [
        &[
            (16u8, 0u8, 0u8),
            (17u8, 6u8, 0u8),
            (18u8, 7u8, 0u8),
            (19u8, 8u8, 0u8),
            (20u8, 9u8, 0u8),
            (21u8, 10u8, 0u8),
            (22u8, 11u8, 0u8),
            (23u8, 13u8, 0u8),
            (24u8, 14u8, 0u8),
        ],
        &[
            (17u8, 0u8, 0u8),
            (18u8, 7u8, 0u8),
            (19u8, 8u8, 0u8),
            (20u8, 9u8, 0u8),
            (21u8, 10u8, 0u8),
            (22u8, 11u8, 0u8),
            (23u8, 12u8, 0u8),
            (24u8, 13u8, 0u8),
        ],
        &[
            (18u8, 0u8, 0u8),
            (19u8, 7u8, 0u8),
            (20u8, 9u8, 0u8),
            (21u8, 10u8, 0u8),
            (22u8, 11u8, 0u8),
            (23u8, 12u8, 0u8),
            (24u8, 13u8, 0u8),
        ],
    ],
    [
        &[
            (17u8, 0u8, 0u8),
            (18u8, 6u8, 0u8),
            (19u8, 7u8, 0u8),
            (20u8, 8u8, 0u8),
            (21u8, 9u8, 0u8),
            (22u8, 10u8, 0u8),
            (23u8, 11u8, 0u8),
            (24u8, 13u8, 0u8),
        ],
        &[
            (18u8, 0u8, 0u8),
            (19u8, 7u8, 0u8),
            (20u8, 8u8, 0u8),
            (21u8, 9u8, 0u8),
            (22u8, 10u8, 0u8),
            (23u8, 11u8, 0u8),
            (24u8, 12u8, 0u8),
        ],
        &[
            (19u8, 0u8, 0u8),
            (20u8, 7u8, 0u8),
            (21u8, 9u8, 0u8),
            (22u8, 10u8, 0u8),
            (23u8, 11u8, 0u8),
            (24u8, 12u8, 0u8),
        ],
    ],
    [
        &[
            (18u8, 0u8, 0u8),
            (19u8, 6u8, 0u8),
            (20u8, 7u8, 0u8),
            (21u8, 8u8, 0u8),
            (22u8, 9u8, 0u8),
            (23u8, 10u8, 0u8),
            (24u8, 11u8, 0u8),
        ],
        &[
            (19u8, 0u8, 0u8),
            (20u8, 7u8, 0u8),
            (21u8, 8u8, 0u8),
            (22u8, 9u8, 0u8),
            (23u8, 10u8, 0u8),
            (24u8, 11u8, 0u8),
        ],
        &[
            (20u8, 0u8, 0u8),
            (21u8, 7u8, 0u8),
            (22u8, 9u8, 0u8),
            (23u8, 10u8, 0u8),
            (24u8, 11u8, 0u8),
        ],
    ],
    [
        &[
            (19u8, 0u8, 0u8),
            (20u8, 6u8, 0u8),
            (21u8, 7u8, 0u8),
            (22u8, 8u8, 0u8),
            (23u8, 9u8, 0u8),
            (24u8, 10u8, 0u8),
            (25u8, 11u8, 0u8),
            (26u8, 13u8, 0u8),
            (27u8, 14u8, 0u8),
            (28u8, 15u8, 0u8),
            (29u8, 16u8, 0u8),
            (30u8, 17u8, 0u8),
            (31u8, 18u8, 0u8),
            (32u8, 19u8, 0u8),
        ],
        &[
            (20u8, 0u8, 0u8),
            (21u8, 7u8, 0u8),
            (22u8, 8u8, 0u8),
            (23u8, 9u8, 0u8),
            (24u8, 10u8, 0u8),
            (25u8, 11u8, 0u8),
            (26u8, 12u8, 0u8),
            (27u8, 13u8, 0u8),
            (28u8, 14u8, 0u8),
            (29u8, 15u8, 0u8),
            (30u8, 16u8, 0u8),
            (31u8, 18u8, 0u8),
            (32u8, 19u8, 0u8),
        ],
        &[
            (21u8, 0u8, 0u8),
            (22u8, 7u8, 0u8),
            (23u8, 9u8, 0u8),
            (24u8, 10u8, 0u8),
            (25u8, 11u8, 0u8),
            (26u8, 12u8, 0u8),
            (27u8, 13u8, 0u8),
            (28u8, 14u8, 0u8),
            (29u8, 15u8, 0u8),
            (30u8, 16u8, 0u8),
            (31u8, 17u8, 0u8),
            (32u8, 18u8, 0u8),
        ],
    ],
    [
        &[
            (20u8, 0u8, 0u8),
            (21u8, 6u8, 0u8),
            (22u8, 7u8, 0u8),
            (23u8, 8u8, 0u8),
            (24u8, 9u8, 0u8),
            (25u8, 10u8, 0u8),
            (26u8, 11u8, 0u8),
            (27u8, 13u8, 0u8),
            (28u8, 14u8, 0u8),
            (29u8, 15u8, 0u8),
            (30u8, 16u8, 0u8),
            (31u8, 17u8, 0u8),
            (32u8, 18u8, 0u8),
        ],
        &[
            (21u8, 0u8, 0u8),
            (22u8, 7u8, 0u8),
            (23u8, 8u8, 0u8),
            (24u8, 9u8, 0u8),
            (25u8, 10u8, 0u8),
            (26u8, 11u8, 0u8),
            (27u8, 12u8, 0u8),
            (28u8, 13u8, 0u8),
            (29u8, 14u8, 0u8),
            (30u8, 15u8, 0u8),
            (31u8, 16u8, 0u8),
            (32u8, 18u8, 0u8),
        ],
        &[
            (22u8, 0u8, 0u8),
            (23u8, 7u8, 0u8),
            (24u8, 9u8, 0u8),
            (25u8, 10u8, 0u8),
            (26u8, 11u8, 0u8),
            (27u8, 12u8, 0u8),
            (28u8, 13u8, 0u8),
            (29u8, 14u8, 0u8),
            (30u8, 15u8, 0u8),
            (31u8, 16u8, 0u8),
            (32u8, 17u8, 0u8),
        ],
    ],
    [
        &[
            (21u8, 0u8, 0u8),
            (22u8, 6u8, 0u8),
            (23u8, 7u8, 0u8),
            (24u8, 8u8, 0u8),
            (25u8, 9u8, 0u8),
            (26u8, 10u8, 0u8),
            (27u8, 11u8, 0u8),
            (28u8, 13u8, 0u8),
            (29u8, 14u8, 0u8),
            (30u8, 15u8, 0u8),
            (31u8, 16u8, 0u8),
            (32u8, 17u8, 0u8),
        ],
        &[
            (22u8, 0u8, 0u8),
            (23u8, 7u8, 0u8),
            (24u8, 8u8, 0u8),
            (25u8, 9u8, 0u8),
            (26u8, 10u8, 0u8),
            (27u8, 11u8, 0u8),
            (28u8, 12u8, 0u8),
            (29u8, 13u8, 0u8),
            (30u8, 14u8, 0u8),
            (31u8, 15u8, 0u8),
            (32u8, 16u8, 0u8),
        ],
        &[
            (23u8, 0u8, 0u8),
            (24u8, 7u8, 0u8),
            (25u8, 9u8, 0u8),
            (26u8, 10u8, 0u8),
            (27u8, 11u8, 0u8),
            (28u8, 12u8, 0u8),
            (29u8, 13u8, 0u8),
            (30u8, 14u8, 0u8),
            (31u8, 15u8, 0u8),
            (32u8, 16u8, 0u8),
        ],
    ],
    [
        &[
            (22u8, 0u8, 0u8),
            (23u8, 6u8, 0u8),
            (24u8, 7u8, 0u8),
            (25u8, 8u8, 0u8),
            (26u8, 9u8, 0u8),
            (27u8, 10u8, 0u8),
            (28u8, 11u8, 0u8),
            (29u8, 13u8, 0u8),
            (30u8, 14u8, 0u8),
            (31u8, 15u8, 0u8),
            (32u8, 16u8, 0u8),
        ],
        &[
            (23u8, 0u8, 0u8),
            (24u8, 7u8, 0u8),
            (25u8, 8u8, 0u8),
            (26u8, 9u8, 0u8),
            (27u8, 10u8, 0u8),
            (28u8, 11u8, 0u8),
            (29u8, 12u8, 0u8),
            (30u8, 13u8, 0u8),
            (31u8, 14u8, 0u8),
            (32u8, 15u8, 0u8),
        ],
        &[
            (24u8, 0u8, 0u8),
            (25u8, 7u8, 0u8),
            (26u8, 9u8, 0u8),
            (27u8, 10u8, 0u8),
            (28u8, 11u8, 0u8),
            (29u8, 12u8, 0u8),
            (30u8, 13u8, 0u8),
            (31u8, 14u8, 0u8),
            (32u8, 15u8, 0u8),
        ],
    ],
];
