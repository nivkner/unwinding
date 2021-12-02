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

#[naked]
pub extern "C-unwind" fn save_context() -> Context {
    // No need to save caller-saved registers here.
    unsafe {
        // use: `str src, [dest, index]`
        // to store values of a register into the context
        // the address of the preallocated stack space is stored in r0
        asm!(
            "
            ret
            ",
            options(noreturn)
        );
    }
}

#[naked]
pub unsafe extern "C" fn restore_context(ctx: &Context) -> ! {
    unsafe {
        // use: `ldr dest, [src, index]`
        // to load values of a register from the context
        // load everything, even the registers that were not saved
        // the arg containing the address to the context is stored in r0
        asm!(
            "
            ret
            ",
            options(noreturn)
        );
    }
}
