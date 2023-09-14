use stm32f0xx_hal::{
    pac::{PWR, RCC, RTC},
    rcc::Rcc,
};

pub struct RealTimeClock {
    rtc: RTC,
    pwr: PWR,
}

impl RealTimeClock {
    pub fn new(pwr: PWR, rtc: RTC) -> Self {
        let mut _self = Self { pwr, rtc };
        _self.configure();
        _self
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

        // -- start configuration by unlocking the registers
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

        // prescaler register
        // LSE clock at 32 768 Hz
        // based on AN4759
        self.rtc
            .prer
            .modify(|_, w| unsafe { w.prediv_a().bits(127).prediv_s().bits(255) });

        self.rtc.isr.modify(|_, w| w.init().clear_bit());

        // -- prepare RTC initialization
        // set INIT bit
        //self.rtc.isr.modify(|_, w| w.init().set_bit());
        // wait for INITF bit to set
        //while self.rtc.isr.read().initf().bit_is_clear() {}

        // configuration register
        /*       self.rtc.cr.modify(|_, w| {
            w
                // hour format: 24h
                .fmt()
                .clear_bit()
        });

        // prescaler register
        // LSE clock at 32 768 Hz
        self.rtc.prer.modify(|_, w| unsafe { w.prediv_a().bits(1) });

        // -- exit initialization mode
        // clear INIT bit
        self.rtc.isr.modify(|_, w| w.init().clear_bit());
        // lock RTC registers
        self.lock();*/
    }

    pub fn get(&self) -> (u8, u8) {
        let sect = self.rtc.tr.read().st().bits();
        let secu = self.rtc.tr.read().su().bits();
        let mint = self.rtc.tr.read().mnt().bits();
        let minu = self.rtc.tr.read().mnu().bits();

        ((10 * mint + minu), (10 * sect + secu))
    }
}
