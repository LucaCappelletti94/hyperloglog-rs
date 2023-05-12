#![no_main]

use arbitrary::Arbitrary;
use hyperloglog_rs::HyperLogLog;
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

fuzz_target!(|data: FuzzCase| {
    let mut idx: u32 = 0;
    let mut left: HyperLogLog<4, 6> = HyperLogLog::new();
    let mut right: HyperLogLog<4, 6> = HyperLogLog::default();
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
            }
            RandomCommand::ResetCounter => {
                idx = 0;
            }
            RandomCommand::Reset => {
                left = HyperLogLog::new();
                right = HyperLogLog::default();
            }
            RandomCommand::FromRegisters => {
                if idx < 1 << 6 {
                    left = HyperLogLog::from_registers(&[idx; 1 << 4]);
                    right = HyperLogLog::from_registers(&[idx; 1 << 4]);    
                }
            }
            RandomCommand::FromCounter => {
                left = HyperLogLog::from(idx);
                right = HyperLogLog::default();
                right.insert(&idx);
            }
        };
    }
});
