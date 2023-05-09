
use crate::prelude::*;

#[inline]
pub fn dispatch_specialized_count<const N: usize>(
    registers: &[u32; N],
) -> (usize, f32) {
    match N {
		4 => count_16(unsafe { core::mem::transmute(registers) }),
		7 => count_32(unsafe { core::mem::transmute(registers) }),
		13 => count_64(unsafe { core::mem::transmute(registers) }),
		26 => count_128(unsafe { core::mem::transmute(registers) }),
		52 => count_256(unsafe { core::mem::transmute(registers) }),
		103 => count_512(unsafe { core::mem::transmute(registers) }),
		205 => count_1024(unsafe { core::mem::transmute(registers) }),
		410 => count_2048(unsafe { core::mem::transmute(registers) }),
		820 => count_4096(unsafe { core::mem::transmute(registers) }),
        _ => unimplemented!(),
    }
}

