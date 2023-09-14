use stm32f0xx_hal::{
    pac::{PWR, RCC, RTC},
    rcc::Rcc,
};

use cortex_m::peripheral::SCB;

pub struct RealTimeClock {
    rtc: RTC,
    pwr: PWR,
    scb: SCB,
}

impl RealTimeClock {
    pub fn new(pwr: PWR, rtc: RTC, scb: SCB) -> Self {
        let mut _self = Self { pwr, rtc, scb };
        _self.configure();
        _self
    }

    pub fn sleep(&mut self) {
        //
        // -- configure WAKEUP pin 2
        self.pwr.csr.modify(|_, w| w.ewup2().set_bit());

        // -- enter STANDBY mode (RM0360 p.83)
        // set SLEEPDEEP in cortex-m0 system control register
        self.scb.set_sleepdeep();
        // set PDDS bit in PWR Control Register
        self.pwr.cr.modify(|_, w| w.pdds().set_bit());
        // clear WUF bit in PWR Control/Status Register
        self.pwr.cr.modify(|_, w| w.cwuf().set_bit());
        // (CWUF is cleared after two system clock cycles)
        cortex_m::asm::delay(100);

        cortex_m::asm::wfe();
    }

    /// Return true if RTC is write protected, false otherwise
    #[inline]
    fn is_write_protected(&self) -> bool {
        // RM0360 p85:
        // DBP: Disable RTC domain write protection
        // 0: Access to RTC disabled
        // 1: Access to RTC enabled
        self.pwr.cr.read().dbp().bit_is_clear()
    }

    /// Unlock RTC registers write protection
    fn unlock(&mut self) {
        // check DBP bit
        if self.is_write_protected() {
            // bit must be set to enable write access
            self.pwr.cr.write(|w| w.dbp().set_bit());
            while self.is_write_protected() {}
        }
    }

    /// Lock RTC registers write protection
    fn lock(&mut self) {
        self.pwr.cr.modify(|_, w| w.dbp().clear_bit());
    }

    fn configure(&mut self) {
        // after domain reset RTC registers are write-protected

        // -- setup
        // enable PWR module
        let apb1enr = unsafe { &(*RCC::ptr()).apb1enr };
        apb1enr.modify(|_, w| w.pwren().enabled());
        //let ahbenr = unsafe { &(*RCC::ptr()).ahbenr };
        //ahbenr.modify(|_, w| w.iopcen().enabled());
        // reset PWR module
        let apb1rstr = unsafe { &(*RCC::ptr()).apb1rstr };
        apb1rstr.write(|w| w.pwrrst().set_bit());
        apb1rstr.write(|w| w.pwrrst().clear_bit());
        //let ahbrstr = unsafe { &(*RCC::ptr()).ahbrstr };
        //ahbrstr.write(|w| w.iopcrst().set_bit());
        //ahbrstr.write(|w| w.iopcrst().clear_bit());

        // -- start RTC configuration by unlocking the registers
        self.unlock();

        let bdcr = unsafe { &(*RCC::ptr()).bdcr };
        // -- configure and start LSE oscillator
        bdcr.write(|w| w.lseon().on().lsedrv().high());
        // wait until LSE clock as started and is stable
        // NOTE this may take a long time
        while bdcr.read().lserdy().is_not_ready() {}

        // write magic sequence to WPR
        self.rtc.wpr.write(|w| unsafe { w.key().bits(0xCA) });
        self.rtc.wpr.write(|w| unsafe { w.key().bits(0x53) });

        // -- power on RTC and configure clock source
        bdcr.modify(|_, w| {
            w
                // enable RTC
                .rtcen()
                .enabled()
                // use LSE oscillator as clock source
                .rtcsel()
                .lse()
        });

        // -- prepare RTC initialization
        // set INIT bit
        self.rtc.isr.modify(|_, w| w.init().set_bit());
        // wait for INITF bit to set
        while self.rtc.isr.read().initf().bit_is_clear() {}

        // prescaler register
        // LSE clock at 32 768 Hz
        // based on AN4759
        self.rtc
            .prer
            .modify(|_, w| unsafe { w.prediv_a().bits(127).prediv_s().bits(255) });

        // configuration date and time
        self.rtc.tr.write(|w| {
            unsafe {
                w
                    // hour format: 24h
                    .pm()
                    .clear_bit()
                    //
                    .ht()
                    .bits(2)
                    .hu()
                    .bits(3)
                    .mnt()
                    .bits(4)
                    .mnt()
                    .bits(2)
            }
        });

        // -- exit initialization mode
        // clear INIT bit
        self.rtc.isr.modify(|_, w| w.init().clear_bit());
        // lock RTC registers
        self.lock();
    }

    pub fn get(&self) -> (u8, u8) {
        let hourt = self.rtc.tr.read().ht().bits();
        let houru = self.rtc.tr.read().hu().bits();
        let mint = self.rtc.tr.read().mnt().bits();
        let minu = self.rtc.tr.read().mnu().bits();

        ((10 * hourt + houru), (10 * mint + minu))
    }
}
