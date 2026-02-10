pub mod drivers;
mod utils;

use drivers::Driver;
use drivers::{JAWS, NVDA, Sapi};

use crate::drivers::TalkError;

#[derive(Clone)]
pub struct Talk {
    inner: Box<dyn Driver>,
}

impl Talk {
    /// Use the NVDA driver
    pub fn nvda() -> Self {
        let inner = Box::new(NVDA::new(None));
        Self { inner }
    }

    /// Use the JAWS driver
    pub fn jaws() -> Result<Self, TalkError> {
        let inner = Box::new(JAWS::new()?);
        Ok(Self { inner })
    }

    /// Use the SAPI driver
    pub fn sapi() -> Self {
        let inner = Box::new(Sapi::new());
        Self { inner }
    }

    /// output text both from speech and braille if supported
    pub fn output(&self, text: &str, interrupt: bool) -> bool {
        self.inner.output(text, interrupt)
    }

    pub fn speak(&self, text: &str, interrupt: bool) -> bool {
        self.inner.speak(text, interrupt)
    }

    pub fn braille(&self, text: &str) -> bool {
        self.inner.braille(text)
    }

    pub fn is_speaking(&self) -> bool {
        self.inner.is_speaking()
    }

    pub fn silence(&self) {
        self.inner.silence();
    }
}
