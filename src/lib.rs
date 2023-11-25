pub mod drivers;
mod utils;

use drivers::Driver;
use drivers::{Sapi, JAWS, NVDA};
use once_cell::sync::OnceCell;
use send_wrapper::SendWrapper;

type DriverQueue = SendWrapper<Vec<Box<dyn Driver>>>;
static ITEMS: OnceCell<DriverQueue> = OnceCell::new();

pub struct Talk {
    drivers: &'static DriverQueue,
    /// current_driver, we'll actually use the index
    current_driver: Option<usize>,
}

impl Talk {
    fn load(prefer_sapi: bool) -> &'static DriverQueue {
        ITEMS.get_or_init(move || {
            let mut drivers: Vec<Box<dyn Driver>> = Vec::new();
            drivers.push(Box::new(Sapi::new()));
            drivers.push(Box::new(JAWS::new()));
            drivers.push(Box::new(NVDA::new(None)));
            // if we don't prefer sapi, move it to last.

            if !prefer_sapi {
                let len = drivers.len();
                drivers.swap(0, len - 1);
            }

            SendWrapper::new(drivers)
        })
    }

    pub fn new() -> Self {
        let drivers = Self::load(false);
        let current_driver = (&**drivers).iter().position(|x| x.is_active() == true);
        Self {
            drivers,
            current_driver,
        }
    }

    /// detect the screen reader in use
    pub fn detect_screen_reader(&self) -> Option<&'static str> {
        for driver in &**self.drivers {
            if !driver.is_active() {
                continue;
            }
            return Some(driver.name());
        }
        None
    }

    pub fn speak(&self, text: &str, interrupt: bool) -> bool {
        (&**self.drivers)
            .get(self.current_driver.unwrap())
            .unwrap()
            .speak(text, interrupt)
    }

    pub fn braille(&self, text: &str) -> bool {
        (&**self.drivers)
            .get(self.current_driver.unwrap())
            .unwrap()
            .braille(text)
    }

    pub fn output(&self, text: &str, interrupt: bool) -> bool {
        self.speak(text, interrupt) || self.braille(text)
    }

    pub fn silence(&self) {
        (&**self.drivers)
            .get(self.current_driver.unwrap())
            .unwrap()
            .silence();
    }
}
