#![no_std]
#![no_main]
// Jackson Coxson
// Fun little game for arduino uno, watch the lights and copy the pattern
// 6 lights, 4 buttons

use panic_halt as _;

use avr_device::interrupt;
use core::cell::RefCell;
use rand::{rngs::SmallRng, Rng, SeedableRng};

mod button;
mod led;

type Console = arduino_hal::hal::usart::Usart0<arduino_hal::DefaultClock>;
static CONSOLE: interrupt::Mutex<RefCell<Option<Console>>> =
    interrupt::Mutex::new(RefCell::new(None));

macro_rules! println {
    ($($t:tt)*) => {
        interrupt::free(
            |cs| {
                if let Some(console) = CONSOLE.borrow(cs).borrow_mut().as_mut() {
                    let _ = ufmt::uwriteln!(console, $($t)*);
                }
            },
        )
    };
}

fn put_console(console: Console) {
    interrupt::free(|cs| {
        *CONSOLE.borrow(cs).borrow_mut() = Some(console);
    })
}

#[derive(Default)]
struct GuessGame {
    pub answer: [u8; 16],
    pub guesses: [u8; 16],
    pub len: u8,
    pub entered: usize,
}

impl GuessGame {
    pub fn new_code(len: u8, rng: &mut SmallRng) -> Self {
        let mut answer = [0_u8; 16];
        for i in 0..len {
            let num = rng.gen_range(0..4);
            answer[i as usize] = num;
        }
        println!("New code: {:?}", answer);
        Self {
            answer,
            guesses: Default::default(),
            len,
            entered: 0,
        }
    }

    pub fn enter_guess(&mut self, guess: u8) -> Option<bool> {
        self.guesses[self.entered] = guess;
        self.entered += 1;
        if self.entered == self.len as usize {
            // Check if the answer is right
            println!("Entered code: {:?}", self.guesses);
            Some(self.is_correct())
        } else {
            None
        }
    }

    pub fn erase_guess(&mut self) {
        self.entered = 0;
        self.guesses = [0_u8; 16];
    }

    pub fn is_correct(&self) -> bool {
        for i in 0..self.len as usize {
            if self.answer[i] != self.guesses[i] {
                return false;
            }
        }
        true
    }

    pub fn flash_sequence(&self, leds: &mut led::GameLeds, delay: u16) {
        for i in 0..self.len as usize {
            leds.blink(self.answer[i] as usize, delay);
            arduino_hal::delay_ms(200)
        }
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // Set up serial console
    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    put_console(serial);
    println!("Hello from main!");

    // Set up pins
    let mut correct_pin = pins.d13.into_output();
    let mut incorrect_pin = pins.d12.into_output();
    let mut game_pins = led::GameLeds::new([
        pins.d2.into_output().downgrade(),
        pins.d3.into_output().downgrade(),
        pins.d4.into_output().downgrade(),
        pins.d5.into_output().downgrade(),
    ]);
    let button_pins = [
        pins.d8.into_pull_up_input().downgrade(),
        pins.d9.into_pull_up_input().downgrade(),
        pins.d10.into_pull_up_input().downgrade(),
        pins.d11.into_pull_up_input().downgrade(),
    ];
    let mut button_pins = button_pins.map(button::ButtonPin::new);

    // Set up RNG
    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());
    let analog_pin = pins.a0.into_analog_input(&mut adc);
    let noise = analog_pin.analog_read(&mut adc) as u64;
    let noise = noise.pow(noise as u32) - noise + 1;
    println!("Seed: {:?}", noise);
    let mut rng = rand::rngs::SmallRng::seed_from_u64(noise);

    // Fancy
    for _ in 0..8 {
        game_pins.wave();
    }
    arduino_hal::delay_ms(1000);

    // Game setup
    let mut idle = 0;
    let mut len = 4;
    let mut delay = 500;
    let mut rounds = 5;
    let mut game = GuessGame::new_code(len, &mut rng);
    game.flash_sequence(&mut game_pins, delay);

    loop {
        // Check if the button is pressed
        for i in 0..4_u8 {
            if button_pins[i as usize].is_pressed() {
                idle = 0;
                game_pins.blink(i as usize, 200);
                if let Some(b) = game.enter_guess(i) {
                    arduino_hal::delay_ms(300);
                    if b {
                        // Flash the green
                        println!("Correct answer");
                        correct_pin.set_high();
                        arduino_hal::delay_ms(1000);
                        correct_pin.set_low();
                        arduino_hal::delay_ms(500);

                        rounds -= 1;
                        if rounds == 0 {
                            rounds = 5;
                            len += 1;
                            if delay > 20 {
                                delay -= 80;
                            }
                            for _ in 0..8 {
                                game_pins.wave();
                            }

                            // End game sequence
                            if len == 15 {
                                loop {
                                    game_pins.wave();
                                    arduino_hal::delay_ms(100);
                                }
                            }
                        }

                        // New game
                        game = GuessGame::new_code(len, &mut rng);
                    } else {
                        // Flash the red
                        println!("Incorrect answer");
                        incorrect_pin.set_high();
                        arduino_hal::delay_ms(1000);
                        incorrect_pin.set_low();
                        arduino_hal::delay_ms(500);
                        game.erase_guess();
                    }
                    game.flash_sequence(&mut game_pins, delay)
                }
            }
        }
        idle += 1;
        if idle > 1000 {
            game_pins.wave();
            idle = 0;
        }
        arduino_hal::delay_ms(10);
    }
}
