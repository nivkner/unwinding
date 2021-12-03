use core::arch::asm;
use core::fmt;
use core::ops;
use gimli::{Arm, Register};

// Match DWARF_FRAME_REGISTERS in libgcc
pub const MAX_REG_RULES: usize = 107;

const GP_REGS: u16 = 16;
const GP_LAST_REG_NUM: u16 = GP_REGS - 1;
const FP_REGS: u16 = 32;
const FP_REG_NUM_OFFSET: u16 = 256;
const FP_LAST_REG_NUM: u16 = FP_REG_NUM_OFFSET + FP_REGS - 1;

#[repr(C)]
#[derive(Clone, Default)]
pub struct Context {
    pub gp: [usize; GP_REGS as usize],
    pub fp: [usize; FP_REGS as usize],
}

impl fmt::Debug for Context {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut fmt = fmt.debug_struct("Context");
        for i in 0..GP_REGS {
            fmt.field(
                Arm::register_name(Register(i)).unwrap(),
                &self.gp[i as usize],
            );
        }
        for i in 0..FP_REGS {
            fmt.field(
                Arm::register_name(Register(i + FP_REG_NUM_OFFSET)).unwrap(),
                &self.fp[i as usize],
            );
        }
        fmt.finish()
    }
}

impl ops::Index<Register> for Context {
    type Output = usize;

    fn index(&self, reg: Register) -> &usize {
        match reg {
            Register(0..=GP_LAST_REG_NUM) => &self.gp[reg.0 as usize],
            Register(FP_REG_NUM_OFFSET..=FP_LAST_REG_NUM) => {
                &self.fp[(reg.0 - FP_REG_NUM_OFFSET) as usize]
            }
            _ => unimplemented!(),
        }
    }
}

impl ops::IndexMut<gimli::Register> for Context {
    fn index_mut(&mut self, reg: Register) -> &mut usize {
        match reg {
            Register(0..=GP_LAST_REG_NUM) => &mut self.gp[reg.0 as usize],
            Register(FP_REG_NUM_OFFSET..=FP_LAST_REG_NUM) => {
                &mut self.fp[(reg.0 - FP_REG_NUM_OFFSET) as usize]
            }
            _ => unimplemented!(),
        }
    }
}

macro_rules! save_regs {
    (gp) => {
        "
        str r4, [r0, 0x4*4]
        str r5, [r0, 0x4*5]
        str r6, [r0, 0x4*6]
        str r7, [r0, 0x4*7]
        str r8, [r0, 0x4*8]
        str r9, [r0, 0x4*9]
        str r10, [r0, 0x4*10]
        str r11, [r0, 0x4*11]
        str r13, [r0, 0x4*13]
        str r14, [r0, 0x4*14]
        "
    };
    (fp) => {
        "
        vstr s16, [r0, 0x4*({fp_offset}+16)]
        vstr s17, [r0, 0x4*({fp_offset}+17)]
        vstr s18, [r0, 0x4*({fp_offset}+18)]
        vstr s19, [r0, 0x4*({fp_offset}+19)]
        vstr s20, [r0, 0x4*({fp_offset}+20)]
        vstr s21, [r0, 0x4*({fp_offset}+21)]
        vstr s22, [r0, 0x4*({fp_offset}+22)]
        vstr s23, [r0, 0x4*({fp_offset}+23)]
        vstr s24, [r0, 0x4*({fp_offset}+24)]
        vstr s25, [r0, 0x4*({fp_offset}+25)]
        vstr s26, [r0, 0x4*({fp_offset}+26)]
        vstr s27, [r0, 0x4*({fp_offset}+27)]
        vstr s28, [r0, 0x4*({fp_offset}+28)]
        vstr s29, [r0, 0x4*({fp_offset}+29)]
        vstr s30, [r0, 0x4*({fp_offset}+30)]
        vstr s31, [r0, 0x4*({fp_offset}+31)]
        "
    };
}

macro_rules! restore_regs {
    (gp) => {
        "
        ldr r4, [r0, 0x4*4]
        ldr r5, [r0, 0x4*5]
        ldr r6, [r0, 0x4*6]
        ldr r7, [r0, 0x4*7]
        ldr r8, [r0, 0x4*8]
        ldr r9, [r0, 0x4*9]
        ldr r10, [r0, 0x4*10]
        ldr r11, [r0, 0x4*11]
        ldr r13, [r0, 0x4*13]
        ldr r14, [r0, 0x4*14]
        "
    };
    (fp) => {
        "
        vldr s16, [r0, 0x4*({fp_offset}+16)]
        vldr s17, [r0, 0x4*({fp_offset}+17)]
        vldr s18, [r0, 0x4*({fp_offset}+18)]
        vldr s19, [r0, 0x4*({fp_offset}+19)]
        vldr s20, [r0, 0x4*({fp_offset}+20)]
        vldr s21, [r0, 0x4*({fp_offset}+21)]
        vldr s22, [r0, 0x4*({fp_offset}+22)]
        vldr s23, [r0, 0x4*({fp_offset}+23)]
        vldr s24, [r0, 0x4*({fp_offset}+24)]
        vldr s25, [r0, 0x4*({fp_offset}+25)]
        vldr s26, [r0, 0x4*({fp_offset}+26)]
        vldr s27, [r0, 0x4*({fp_offset}+27)]
        vldr s28, [r0, 0x4*({fp_offset}+28)]
        vldr s29, [r0, 0x4*({fp_offset}+29)]
        vldr s30, [r0, 0x4*({fp_offset}+30)]
        vldr s31, [r0, 0x4*({fp_offset}+31)]
        "
    };
}

#[naked]
pub extern "C-unwind" fn save_context() -> Context {
    // No need to save caller-saved registers here.
    unsafe {
        #[cfg(target_feature = "vfp2")]
        asm!(
            concat!(save_regs!(gp), save_regs!(fp), "bx lr"),
            fp_offset = const GP_REGS,
            options(noreturn)
        );
        #[cfg(not(target_feature = "vfp2"))]
        asm!(concat!(save_regs!(gp), "bx lr"), options(noreturn));
    }
}

#[naked]
pub unsafe extern "C" fn restore_context(ctx: &Context) -> ! {
    unsafe {
        #[cfg(target_feature = "vfp2")]
        asm!(
            concat!(restore_regs!(gp), restore_regs!(fp), "bx lr"),
            fp_offset = const GP_REGS,
            options(noreturn)
        );
        #[cfg(not(target_feature = "vfp2"))]
        asm!(concat!(restore_regs!(gp), "bx lr"), options(noreturn));
    }
}
