mod jaws;
mod nvda;
mod sapi;
pub use self::jaws::JAWS;
pub use self::nvda::NVDA;
pub use self::sapi::Sapi;

use dyn_clone::DynClone;
use thiserror::Error;

/// The Driver trait
///
/// This trait defines the interface of a screen reader driver
/// It will be used to query info and interact with the underlying library instance
/// Every method except for `speak` has a default implementation
/// This allows for flexible customization and less repetitive code for the implementors of this trait, as screen readers and or TTS may support different features
pub trait Driver: DynClone {
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
    fn is_active(&self) -> bool {
        false
    }
}

/// Message used to interact with background thread workers that will interact with the specific driver
/// Useful in cases where the only communication mechanism is for example COM
#[derive(Debug)]
pub enum Command {
    Speak(String, bool),
    Braille(String),
    Output(String, bool),
    IsSpeaking(oneshot::Sender<bool>),
    Silence,
    /// used to shutdown the background thread loop
    /// Useful in Drop contexts
    Shutdown,
    IsActive(oneshot::Sender<bool>),
}

/// General library error
#[derive(Debug, Error)]
pub enum TalkError {
    /// The initialization of a driver has failed
    #[error("Driver initialization has failed")]
    DriverInitializationError,
}

dyn_clone::clone_trait_object!(Driver);
