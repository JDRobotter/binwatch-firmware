#![no_main]
#![no_std]

mod display;
use display::WatchDisplay;

mod rtc;
use rtc::RealTimeClock;

use cortex_m::interrupt::Mutex;

// Setup startup code and minimal runtime for uC
// (check https://docs.rs/cortex-m-rt/latest/cortex_m_rt/)
use cortex_m_rt::entry;

use stm32f0xx_hal::{
    pac::{self, interrupt},
    prelude::*,
};

use irq::{handler, scope, scoped_interrupts};

scoped_interrupts! {
    enum Interrupt {
        TIM3,
    }

    use #[interrupt];
}

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

    // setup RTC
    cortex_m::asm::delay(100_000);
    let rtc = RealTimeClock::new(p.PWR, p.RTC);
    // setup display
    let mut display = WatchDisplay::new(p.GPIOA, p.GPIOB);

    //// XXX
    display.set_time(0xff, 0xff);
    cortex_m::asm::delay(100000);
    display.set_time(0, 0);
    // XXX

    handler!(tim3_handler = || {});

    scope(|scope| {
        scope.register(Interrupt::TIM3, tim3_handler);

        let mut _nvic = cp.NVIC;
        unsafe {
            cortex_m::peripheral::NVIC::unmask(pac::Interrupt::TIM3);
        }

        let mut k = 0u32;
        loop {
            k += 1;

            if k % 10000 == 0 {
                let (mins, secs) = rtc.get();
                display.set_time(mins, secs);
            }

            display.update();
        }
    });

    // not reachable
    panic!();
}
