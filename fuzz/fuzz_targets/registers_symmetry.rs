#![no_main]

use arbitrary::Arbitrary;
use hyperloglog_rs::prelude::*;
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct FuzzCase {
    commands: Vec<RandomCommand>,
}

#[derive(Arbitrary, Debug)]
enum RandomCommand {
    Reset,
    ResetCounter,
    Multiply,
    Divide,
    IncreaseCounter,
    InsertCounter,
    FromCounter,
    FromRegisters,
}

const BITS: usize = 5;

fuzz_target!(|data: FuzzCase| {
    let mut idx: u32 = 0;
    let mut left: HyperLogLog<Precision8, BITS> = HyperLogLog::new();
    let mut right: HyperLogLog<Precision8, BITS> = HyperLogLog::default();
    for command in data.commands {
        match command {
            RandomCommand::Multiply => {
                idx = idx.wrapping_mul(2);
            }
            RandomCommand::Divide => {
                idx /= 2;
            }
            RandomCommand::IncreaseCounter => {
                idx = idx.wrapping_add(1);
            }
            RandomCommand::InsertCounter => {
                let old_left_registers = left.get_registers();
                let old_right_registers = right.get_registers();
                let previous = left.estimate_cardinality();
                left.insert(&idx);
                right.insert(&idx);
                assert!(left.may_contain(&idx));
                assert!(right.may_contain(&idx));
                assert_eq!(left, right);
                assert!(
                    !f32::is_nan(previous),
                    "Estimation should never be NaN, but got {}. Specifically, the registers were {:?}",
                    previous,
                    left.get_registers()
                );
                assert_eq!(left.clone() | right.clone(), right);
                assert_eq!(left.clone() | right.clone(), left);
                assert!(
                    previous <= left.estimate_cardinality(),
                    "Estimate should never decrease, but it did. Previous: {}, current: {}",
                    previous,
                    left.estimate_cardinality()
                );
                let left_registers = left.get_registers();
                let right_registers = right.get_registers();

                // We check that all values in the left and right registers
                // are equal or larger than the old values.
                assert!(
                    left_registers
                        .iter()
                        .zip(old_left_registers.iter())
                        .all(|(new, old)| new >= old),
                    "Left registers should be equal or larger than the old left registers. Left: {:?}, old left: {:?}",
                    left_registers,
                    old_left_registers
                );

                assert!(
                    right_registers
                        .iter()
                        .zip(old_right_registers.iter())
                        .all(|(new, old)| new >= old),
                    "Right registers should be equal or larger than the old right registers. Right: {:?}, old right: {:?}",
                    right_registers,
                    old_right_registers
                );

                assert_eq!(
                    left_registers, right_registers,
                    "Registers should be equal, but they are not. Left: {:?}, right: {:?}",
                    left_registers, right_registers
                );
                let restored_left = HyperLogLog::from_registers(&left_registers);
                let restored_right = HyperLogLog::from_registers(&right_registers);
                assert_eq!(
                    restored_left, left,
                    "Restored left should be equal to the original left"
                );
                assert_eq!(
                    restored_right, right,
                    "Restored right should be equal to the original right"
                );

                // If there are k registers equal to zero, then the cardinality estimate
                // must be greater of equal to number of registers minus k.

                let left_zero_registers = left_registers.iter().filter(|&&x| x == 0).count();
                let right_zero_registers = right_registers.iter().filter(|&&x| x == 0).count();

                assert!(
                    left.estimate_cardinality() >= ((left.get_number_of_registers()) - left_zero_registers) as f32,
                    "Estimate should be greater or equal to the number of registers minus the number of zero registers. Estimate: {}, number of zero registers: {}, number of registers: {}",
                    left.estimate_cardinality(),
                    left_zero_registers,
                    left.get_number_of_registers()
                );

                assert!(
                    right.estimate_cardinality() >= ((left.get_number_of_registers()) - right_zero_registers) as f32,
                    "Estimate should be greater or equal to the number of registers minus the number of zero registers. Estimate: {}, number of zero registers: {}, number of registers: {}",
                    right.estimate_cardinality(),
                    right_zero_registers,
                    left.get_number_of_registers()
                );
            }
            RandomCommand::ResetCounter => {
                idx = 0;
            }
            RandomCommand::Reset => {
                left = HyperLogLog::new();
                right = HyperLogLog::default();
            }
            RandomCommand::FromRegisters => {
                if idx < 1 << BITS {
                    left = HyperLogLog::from_registers(&[idx; <Precision8 as Precision<BITS>>::NUMBER_OF_REGISTERS]);
                    right = HyperLogLog::from_registers(&[idx; <Precision8 as Precision<BITS>>::NUMBER_OF_REGISTERS]);
                }
            }
            RandomCommand::FromCounter => {
                left = HyperLogLog::from(idx);
                right = HyperLogLog::default();
                right.insert(&idx);

                // After having inserted an element
                // the registers should never appear empty.

                assert!(
                    left.get_registers().iter().any(|&x| x != 0),
                    "Left registers should not be empty after having inserted an element"
                );

                assert!(
                    right.get_registers().iter().any(|&x| x != 0),
                    "Right registers should not be empty after having inserted an element"
                );

                assert!(
                    !left.is_empty(),
                    "Left registers should not be empty after having inserted an element"
                );

                assert!(
                    !right.is_empty(),
                    "Right registers should not be empty after having inserted an element"
                );
            }
        };
    }
});
