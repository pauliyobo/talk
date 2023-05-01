mod nvda;
mod sapi;
pub use self::nvda::NVDA;
pub use self::sapi::Sapi;


/// The Driver trait
/// 
/// This trait defines the interface of a screen reader driver
/// It will be used to query info and interact with the underlying library instance

pub trait Driver {
    // the name of the driver
    fn name(&self) -> &'static str;
    // Is the driver speaking?
    fn is_speaking(&self) -> bool;
    fn speak<S: Into<String>>(&self, text: S, interrupt: bool) -> bool;
    fn braille<S: Into<String>>(&self, text: S) -> bool;
    fn output(&self, text: &str, interrupt: bool) -> bool {
        self.speak(text, interrupt) || self.braille(text)
    }
    fn silence(&self) -> bool;
}