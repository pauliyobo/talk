//! Sapi 5.3 Driver
use crate::drivers::Driver;
use crate::utils::to_bstr;
use windows::{Win32::Media::Speech::*, Win32::System::Com::*};

pub struct Sapi(ISpeechVoice);

impl Sapi {
    pub fn new() -> Self {
        unsafe { CoInitialize(None).unwrap() };
        let voice: ISpeechVoice = unsafe {
            CoCreateInstance(&SpVoice, None, CLSCTX_ALL).expect("Could not initialize Sapi voice.")
        };
        Sapi(voice)
    }
}

impl Default for Sapi {
    fn default() -> Self {
        Self::new()
    }
}

impl Driver for Sapi {
    fn name(&self) -> &'static str {
        "Sapi"
    }

    fn speak<S: Into<String>>(&self, text: S, interrupt: bool) -> bool {
        let text: String = text.into();
        let bstr = to_bstr(&text).unwrap();
        // SVSFlagsAsync does not seem to work
        let mut flags: SpeechVoiceSpeakFlags = SVSFIsNotXML;
        if interrupt {
            flags.0 |= SVSFPurgeBeforeSpeak.0;
        }
        unsafe { self.0.Speak(&bstr, flags).is_ok() }
    }
}
