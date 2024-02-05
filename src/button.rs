// Jackson Coxson


use arduino_hal::{hal::port::Dynamic, port::{mode::{Input, PullUp}, Pin}};

pub enum ButtonPinState {
    Rizzen,
    Fallen
}

pub struct ButtonPin {
    state: ButtonPinState,
    pin: Pin<Input<PullUp>, Dynamic>
}

impl ButtonPin {
    pub fn is_pressed(&mut self) -> bool {
        if self.pin.is_low() {
            self.state = ButtonPinState::Rizzen;
            false
        } else {
            match self.state {
                ButtonPinState::Rizzen => {
                    self.state = ButtonPinState::Fallen;
                    true
                }
                ButtonPinState::Fallen => false
            }
        }
    }

    pub fn new(pin: Pin<Input<PullUp>, Dynamic>) -> Self {
        ButtonPin { state: ButtonPinState::Fallen, pin }
    }
}