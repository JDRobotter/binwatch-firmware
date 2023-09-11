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
        self.pwr.cr.read().dbp().bit()
    }

    /// Unlock RTC registers write protection
    fn unlock(&mut self) {
        // disable RTC domain write protection
        if self.pwr.cr.read().dbp().bit_is_set() {
            self.pwr.cr.modify(|_, w| w.dbp().set_bit());
            while self.pwr.cr.read().dbp().bit_is_clear() {}
        }
    }

    /// Lock RTC registers write protection
    fn lock(&mut self) {
        self.pwr.cr.modify(|_, w| w.dbp().clear_bit());
    }

    fn configure(&mut self) {
        // after domain reset RTC registers are write-protected
        // -- start configuration by unlocking the registers
        self.unlock();

        // -- stup
        // reset PWR module
        let apb1rstr = unsafe { &(*RCC::ptr()).apb1rstr };
        apb1rstr.write(|w| w.pwrrst().set_bit());
        apb1rstr.write(|w| w.pwrrst().clear_bit());

        // enable PWR module
        let apb1enr = unsafe { &(*RCC::ptr()).apb1enr };
        apb1enr.modify(|_, w| w.pwren().enabled());

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
