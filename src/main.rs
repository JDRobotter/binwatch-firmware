#![no_main]
#![no_std]

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

    let gpioa = p.GPIOA.split(&mut rcc);
    let gpiob = p.GPIOB.split(&mut rcc);
    let gpioc = p.GPIOC.split(&mut rcc);

    // setup leds GPIOs
    let (
        mut button,
        mut hour0,
        mut hour1,
        mut hour2,
        mut hour3,
        mut hour4,
        mut min0,
        mut min1,
        mut min2,
        mut min3,
        mut min4,
        mut min5,
    ) = cortex_m::interrupt::free(move |cs| {
        (
            gpioc.pc13.into_pull_down_input(cs),
            gpiob.pb14.into_push_pull_output(cs), // H0
            gpioa.pa7.into_push_pull_output(cs),  // H1
            gpioa.pa5.into_push_pull_output(cs),  // H2
            gpioa.pa2.into_push_pull_output(cs),  // H3
            gpioa.pa0.into_push_pull_output(cs),  // H4
            gpiob.pb13.into_push_pull_output(cs), // M0
            gpiob.pb12.into_push_pull_output(cs), // M1
            gpioa.pa6.into_push_pull_output(cs),  // M2
            gpioa.pa4.into_push_pull_output(cs),  // M3
            gpioa.pa3.into_push_pull_output(cs),  // M4
            gpioa.pa1.into_push_pull_output(cs),  // M5
        )
    });

    // setup RTC
    cortex_m::asm::delay(100_000);
    let rtc = RealTimeClock::new(p.PWR, p.RTC);

    handler!(tim3_handler = || {});

    scope(|scope| {
        scope.register(Interrupt::TIM3, tim3_handler);

        let mut _nvic = cp.NVIC;
        unsafe {
            cortex_m::peripheral::NVIC::unmask(pac::Interrupt::TIM3);
        }

        loop {
            let (mins, secs) = rtc.get();

            if secs & 1 != 0 {
                min0.set_high().ok();
            } else {
                min0.set_low().ok();
            }
            if secs & 2 != 0 {
                min1.set_high().ok();
            } else {
                min1.set_low().ok();
            }
            if secs & 4 != 0 {
                min2.set_high().ok();
            } else {
                min2.set_low().ok();
            }
            if secs & 8 != 0 {
                min3.set_high().ok();
            } else {
                min3.set_low().ok();
            }
            if secs & 16 != 0 {
                min4.set_high().ok();
            } else {
                min4.set_low().ok();
            }
            if secs & 32 != 0 {
                min5.set_high().ok();
            } else {
                min5.set_low().ok();
            }
            cortex_m::asm::delay(100_000);
        }
    });

    // not reachable
    panic!();
}
