// SPDX-License-Identifier: GPL-2.0

//! Driver for Raspi GPIO LED.

#![no_std]
use led_gpio::{RpiGpio,GpioFunction};
// TODO
// 参考 drivers/gpio/exercise-3-gpio/gpio-sample-rpbled.c代码
// C代码里面用到的字符设备，还是用练习二的Msicdev来实现就可以了
// OSLayer已经集成，但是Adapter driver和Pure driver层的代码需要自己实现
// 最好通过树莓派开发板来查看相应的GPIO控制效果
//

use core::ops::Deref;
use core::result::Result::Ok;
use kernel::prelude::*;
use kernel::{
    file::{self, File},
    io_buffer::{IoBufferReader, IoBufferWriter},
    sync::{Arc, ArcBorrow},
    sync::Mutex,
    miscdev, 
    pin_init,
    new_mutex,
    fmt,
};
use kernel::bindings;


module! {
    type: RustMiscDev,
    name: "rust_miscdev_gpio",
    author: "xik",
    description: "Rust exercise 003",
    license: "GPL",
}
// const BCM2837_GPIO_BASE:u32 = 0xFE200000;  // 4b
const BCM2837_GPIO_BASE:u64 = 0x3F200000;  // 3b+
const GLOBALMEM_SIZE: usize = 0x1000;
const GPIO_SIZE:u64 = 0xB4;

#[pin_data]
struct RustMiscdevData {
    #[pin]
    inner: Mutex<[u8;GLOBALMEM_SIZE]>,
    #[pin]
    rpi_gpio: Mutex<RpiGpio>
}

impl RustMiscdevData {
    fn try_new() -> Result<Arc<Self>>{
        pr_info!("rust miscdevice for gpio created\n");
        let mapped_base = unsafe { bindings::ioremap(BCM2837_GPIO_BASE, GPIO_SIZE) };
        Ok(Arc::pin_init(
            pin_init!(Self {
                inner <- new_mutex!([0u8;GLOBALMEM_SIZE]),
                rpi_gpio <- new_mutex!(RpiGpio::new(mapped_base as *mut u8))
            })
        )?)
    }
}

unsafe impl Sync for RustMiscdevData {}
unsafe impl Send for RustMiscdevData {}


// unit struct for file operations
struct RustFile;
#[vtable]
impl file::Operations for RustFile {
    type Data = Arc<RustMiscdevData>;
    type OpenData = Arc<RustMiscdevData>;

    fn open(_shared: &Arc<RustMiscdevData>, _file: &file::File) -> Result<Self::Data> {
        pr_info!("open in miscdevice\n",);
        Ok(Arc::clone(_shared))
    }

    fn read(
        _shared: ArcBorrow<'_, RustMiscdevData>,
        _file: &File,
        _writer: &mut impl IoBufferWriter,
        _offset: u64,
    ) -> Result<usize> {
        pr_info!("read in miscdevice\n");
        
        let inner = _shared.deref().inner.lock();

        let offset = _offset as usize;
        let left_size = inner.len()-offset;
        let mut data_len = _writer.len();
        if left_size < _writer.len(){
            data_len = left_size;
        }
        // write to io buffer
        let _ = _writer.write_slice(&inner[offset..offset+data_len]);
        Ok(data_len)
    }

    fn write(
        _shared: ArcBorrow<'_, RustMiscdevData>,
        _file: &File,
        _reader: &mut impl IoBufferReader,
        _offset: u64,
    ) -> Result<usize> {
        pr_info!("write in miscdevice\n");
        let mut inner = _shared.deref().inner.lock();
        let offset = _offset as usize;
        // not enough place to write
        let reader_len =  _reader.len();
        if GLOBALMEM_SIZE - offset < reader_len {
            return Err(EFAULT);
        }
        // read from io buffer
        let _result = _reader.read_slice(&mut inner[offset..offset+_reader.len()]);
        let mut rpi_gpio = _shared.deref().rpi_gpio.lock();
        match inner[offset] {
            b'0' => {
                let _ = rpi_gpio.set_value(17,0);
            }
            b'1' => {
                let _ = rpi_gpio.set_value(17,1);
            }
            _ => {
                return Err(EINVAL);
            }
        }
        Ok(reader_len)
    }

    fn release(_data: Self::Data, _file: &File) {
        pr_info!("release in miscdevice\n");
    }
}

struct RustMiscDev {
    _dev: Pin<Box<miscdev::Registration<RustFile>>>,
}

impl kernel::Module for RustMiscDev {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust miscdevice device for gpio (init)\n");

        let data: Arc<RustMiscdevData> = RustMiscdevData::try_new()?;
        let _ =  (*data).rpi_gpio.lock().gpio_init();

        let misc_reg = miscdev::Registration::new_pinned(fmt!("rust_misc_gpio"), data)?;

        Ok(RustMiscDev { _dev: misc_reg })
    }
}

impl Drop for RustMiscDev {
    fn drop(&mut self) {
        pr_info!("Rust miscdevice device sample (exit)\n");
    }
}