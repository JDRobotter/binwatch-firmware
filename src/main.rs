#![no_main]
#![no_std]

mod display;
use display::WatchDisplay;

mod rtc;
use rtc::RealTimeClock;

// Setup startup code and minimal runtime for uC
// (check https://docs.rs/cortex-m-rt/latest/cortex_m_rt/)
use cortex_m_rt::entry;

use stm32f0xx_hal::{
    pac::{self, interrupt, PWR},
    prelude::*,
};

use core::panic::PanicInfo;
#[panic_handler]
fn panic(_pi: &PanicInfo) -> ! {
    loop {}
}

#[entry]
fn main() -> ! {
    let mut p = pac::Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    // when booting from bootloader memory mapping will have
    // System Flash memory (bootloader code) mapped at 0x0000_0000
    // resulting in bootloader interrupt vectors to be used
    // The following line will force memory mapping to Main Flash memory
    // as intended by this code
    p.SYSCFG.cfgr1.modify(|_, w| w.mem_mode().main_flash());

    // configure clock frequency
    let mut rcc = p.RCC.configure().sysclk(1.mhz()).freeze(&mut p.FLASH);

    //let gpioa = p.GPIOA.split(&mut rcc);
    //let mut dbg = cortex_m::interrupt::free(|cs| gpioa.pa7.into_push_pull_output(cs));

    //let gpiob = p.GPIOB.split(&mut rcc);
    let gpioc = p.GPIOC.split(&mut rcc);
    let button = cortex_m::interrupt::free(|cs| gpioc.pc13.into_floating_input(cs));
    // setup RTC
    cortex_m::asm::delay(100_000);
    let mut rtc = RealTimeClock::new(p.PWR, p.RTC, cp.SCB);
    // setup display
    let mut display = WatchDisplay::new(p.GPIOA, p.GPIOB);

    let mut k = 0u32;
    loop {
        // check button state
        /*
        if button.is_high().unwrap() {
            // wait button to go low
            while button.is_high().unwrap() {}
            rtc.sleep();
        }*/

        if k % 10000 == 0 {
            let (hours, mins) = rtc.get();
            display.set_time(hours, mins);
        }

        if k > 500_000 {
            rtc.sleep();
        }

        display.update();
        k += 1;
    }
}
