use stm32f0xx_hal::pac::{GPIOA, GPIOB, RCC};

// STM32 leds port assignations
// H0: pb14
// H1: pa7
// H2: pa5
// H3: pa2
// H4: pa0
// M0: pb13
// M1: pb12
// M2: pa6
// M3: pa4
// M4: pa3
// M5: pa1

const MASKS_HOURS: [(u32, u32); 5] = [
    (0, 1 << 14), // H0
    (1 << 7, 0),  // H1
    (1 << 5, 0),  // H2
    (1 << 2, 0),  // H3
    (1 << 0, 0),  // H4
];

const MASKS_MINS: [(u32, u32); 6] = [
    (0, 1 << 13), // M0
    (0, 1 << 12), // M1
    (1 << 6, 0),  // M2
    (1 << 4, 0),  // M3
    (1 << 3, 0),  // M4
    (1 << 1, 0),  // M5
];

pub struct WatchDisplay {
    gpioa: GPIOA,
    gpiob: GPIOB,
    display: (u32, u32),
    time: u16,
}

impl WatchDisplay {
    pub fn new(gpioa: GPIOA, gpiob: GPIOB) -> Self {
        let mut _self = Self {
            gpioa,
            gpiob,
            display: (0, 0),
            time: 0,
        };

        _self.configure_gpios();

        _self
    }

    fn make_mask<const N: usize>(masks: &[(u32, u32); N], byte: u8) -> (u32, u32) {
        masks
            .iter()
            .enumerate()
            .map(|(k, (wa, wb))| {
                // check if bit is set in input value
                if byte & (1 << k) != 0 {
                    (*wa, *wb)
                } else {
                    (0, 0)
                }
            })
            .fold((0, 0), |(awa, awb), (wa, wb)| (awa | wa, awb | wb))
    }

    fn make_hour_mins_mask(hours: u8, mins: u8) -> (u32, u32) {
        let (hwa, hwb) = Self::make_mask(&MASKS_HOURS, hours);
        let (mwa, mwb) = Self::make_mask(&MASKS_MINS, mins);

        let wa = hwa | mwa;
        let wb = hwb | mwb;

        (wa, wb)
    }

    fn shift_mask_to_2_bits(mask: u32, value_when_set: u8) -> u32 {
        let mut omask = 0u32;
        for k in 0..16 {
            if (mask & (1 << k)) != 0 {
                omask |= (value_when_set as u32) << (2 * k);
            }
        }
        omask
    }

    fn configure_gpios(&mut self) {
        // fetch mask on port A and B
        let (wa, wb) = Self::make_hour_mins_mask(0xff, 0xff);

        // turn on GPIOA & B clocks
        let ahbenr = unsafe { &(*RCC::ptr()).ahbenr };
        ahbenr.modify(|_, w| w.iopaen().enabled().iopben().enabled());

        // -- configure GPIOs
        // build mask (each port in MODER is 2 bits wide)
        // 0b01 is general purpose output mode
        let mask = Self::shift_mask_to_2_bits(wa, 0b01);
        self.gpioa.moder.write(|w| unsafe { w.bits(mask) });
        // 0b00 is low speed
        let mask = Self::shift_mask_to_2_bits(wa, 0b00);
        self.gpioa.ospeedr.write(|w| unsafe { w.bits(mask) });
        // 0b01 is pull-up
        let mask = Self::shift_mask_to_2_bits(wa, 0b01);
        self.gpioa.pupdr.write(|w| unsafe { w.bits(mask) });
        // configure port as open-drain
        self.gpioa.otyper.write(|w| unsafe { w.bits(wa) });

        // build mask (each port in MODER is 2 bits wide)
        // 0b01 is general purpose output mode
        let mask = Self::shift_mask_to_2_bits(wb, 0b01);
        self.gpiob.moder.write(|w| unsafe { w.bits(mask) });
        // 0b00 is low speed
        let mask = Self::shift_mask_to_2_bits(wb, 0b00);
        self.gpiob.ospeedr.write(|w| unsafe { w.bits(mask) });
        // 0b01 is pull-up
        let mask = Self::shift_mask_to_2_bits(wb, 0b01);
        self.gpiob.pupdr.write(|w| unsafe { w.bits(mask) });
        // configure port as open-drain
        self.gpiob.otyper.write(|w| unsafe { w.bits(wb) });
    }

    pub fn set_time(&mut self, hours: u8, mins: u8) {
        self.display = Self::make_hour_mins_mask(hours, mins);
    }

    pub fn update(&mut self) {
        self.time = u16::wrapping_add(self.time, 1);

        let (wa, wb) = if self.time % 20 == 0 {
            (0xffff, 0xffff)
        } else {
            self.display
        };

        self.gpioa.odr.write(|w| unsafe { w.bits(wa) });
        self.gpiob.odr.write(|w| unsafe { w.bits(wb) });
    }
}
