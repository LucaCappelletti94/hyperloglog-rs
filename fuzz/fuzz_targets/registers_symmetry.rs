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
    IncreaseCounter,
    InsertCounter,
    FromCounter,
}

fuzz_target!(|data: FuzzCase| {
    let mut idx = 0;
    let mut left: HyperLogLog<8, 8> = HyperLogLog::new();
    let mut right: HyperLogLog<8, 8> = HyperLogLog::default();
    for command in data.commands {
        match command {
            RandomCommand::IncreaseCounter => {
                idx += 1;
            }
            RandomCommand::InsertCounter => {
                let previous = left.estimate_cardinality();
                left.insert(&idx);
                right.insert(&idx);
                assert!(left.may_contain(&idx));
                assert!(right.may_contain(&idx));
                assert_eq!(left, right);
                assert_eq!(left.clone() | right.clone(), right);
                assert_eq!(left.clone() | right.clone(), left);
                assert!(previous <= left.estimate_cardinality());
            }
            RandomCommand::Reset => {
                left = HyperLogLog::new();
                right = HyperLogLog::default();
            }
            RandomCommand::FromCounter => {
                left = HyperLogLog::from(idx);
                right = HyperLogLog::default();
                right.insert(&idx);
            }
        };
    }
});
