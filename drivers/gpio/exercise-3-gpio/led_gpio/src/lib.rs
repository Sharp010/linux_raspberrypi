// pure driver

//! Driver for the Led GPIO
//!

#![no_std]
#![feature(const_ptr_as_ref)]
#![feature(const_option)]
#![feature(const_nonnull_new)]


use core::ptr::NonNull;
pub(crate) use osl::error::Result;
const __LOG_PREFIX: &[u8] = b"gpio\0";
use osl::log_info;
use tock_registers::{
    interfaces::{Readable, Writeable},
    registers::{ReadWrite,ReadOnly,WriteOnly},
};


#[repr(C)]
#[allow(non_snake_case)]
pub(crate) struct RPiGpioRegisters {
    pub(crate) GPFSEL: [ReadWrite<u32>; 6],    // 0x00
    reserved: u32,
    pub(crate) GPSET: [WriteOnly<u32>; 2],     // 0x1c
    reserved1: u32,
    pub(crate) GPCLR: [WriteOnly<u32>; 2],     // 0x28
    reserved2: u32,
    pub(crate) GPLEV: [ReadOnly<u32>; 2],      // 0x34
    reserved3: u32,
    pub(crate) GPEDS: [ReadWrite<u32>;2],      // 0x40
    reserved4: u32,
    pub(crate) GPREN: [ReadWrite<u32>;2],      // 0x4C
    reserved5: u32,
    pub(crate) GPFEN: [ReadWrite<u32>;2],      // 0x58
    reserved6: u32,
    pub(crate) GPHEN: [ReadWrite<u32>;2],      // 0x64
    reserved7: u32,
    pub(crate) GPLEN: [ReadWrite<u32>;2],      // 0x70
    reserved8: u32,
    pub(crate) GPAREN: [ReadWrite<u32>;2],      // 0x7C
    reserved9: u32,
    pub(crate) GPAFEN: [ReadWrite<u32>;2],      // 0x88
    reserved10: u32,
    pub(crate) GPPUD:  ReadWrite<u32>,         // 0x94
    pub(crate) GPPUDCLK: [ReadWrite<u32>;2],      // 0x98
    reserved11: u32,
    test:char
}



#[derive(Copy, Clone)]
pub struct RpiGpioPort {
    regs: NonNull<RPiGpioRegisters>,
}

unsafe impl Send for RpiGpioPort {}
unsafe impl Sync for RpiGpioPort {}

impl RpiGpioPort {
    pub const fn new(base_addr: *mut u8) -> Self {
        
        Self {
            regs: NonNull::new(base_addr).unwrap().cast(),
        }
    }

    const fn regs(&self) -> &RPiGpioRegisters {
        unsafe { self.regs.as_ref() }
    }

    /// 
    pub fn direction_output(&mut self, offset: u32) -> Result<()> {
        let fsel_index = offset as usize / 10;
        let fsel_shift = (offset % 10) * 3;
        let mut fsel_value = self.regs().GPFSEL[fsel_index].get();
        // 保持其他位不变
        // 对应引脚的功能选择位变成001
        fsel_value &= !(0b111 << fsel_shift);
        fsel_value |= 0b001 << fsel_shift; 
        self.regs().GPFSEL[fsel_index].set(fsel_value);
        Ok(())
    }

    /// 
    pub fn set_value(&mut self, offset: u32, value: u32) -> Result<()> {
        if value == 1 {
            self.regs().GPSET[offset as usize / 32].set(1 << (offset % 32));
        } else {
            self.regs().GPCLR[offset as usize / 32].set(1 << (offset % 32));
        }
        Ok(())
    }
}