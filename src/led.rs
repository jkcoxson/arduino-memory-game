// Jackson Coxson

use arduino_hal::{hal::port::Dynamic, port::{mode::Output, Pin}};

pub struct GameLeds([Pin<Output, Dynamic>; 4]);

impl GameLeds {
    pub fn new(pins: [Pin<Output, Dynamic>; 4]) -> Self {
        Self(pins)
    }

    pub fn wave(&mut self) {
        for led in self.0.iter_mut() {
            led.set_high();
            arduino_hal::delay_ms(50);
        }
        for led in self.0.iter_mut() {
            led.set_low();
            arduino_hal::delay_ms(50);
        }
    }

    pub fn blink(&mut self, led: usize, duration: u16) {
        self.0[led].set_high();
        arduino_hal::delay_ms(duration);
        self.0[led].set_low();
    }
}