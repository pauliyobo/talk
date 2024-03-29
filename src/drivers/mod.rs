mod jaws;
mod nvda;
mod sapi;
pub use self::jaws::JAWS;
pub use self::nvda::NVDA;
pub use self::sapi::Sapi;

/// The Driver trait
///
/// This trait defines the interface of a screen reader driver
/// It will be used to query info and interact with the underlying library instance
/// Every method except for `speak` have a default implementation
/// This allows for flexible customization and less repetitive code for the implementors of this trait, as screen readers and or TTS may support different features
pub trait Driver {
    /// the name of the driver
    fn name(&self) -> &'static str;

    /// Is the driver speaking?
    fn is_speaking(&self) -> bool {
        false
    }

    fn speak(&self, text: &str, interrupt: bool) -> bool;

    fn braille(&self, _text: &str) -> bool {
        false
    }

    fn output(&self, text: &str, interrupt: bool) -> bool {
        self.speak(text, interrupt) || self.braille(text)
    }

    fn silence(&self) {}

    /// specifies whether the driver is active
    fn is_active(&self) -> bool;
}
