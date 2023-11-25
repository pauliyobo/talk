//! Sapi 5.3 Driver
use crate::drivers::Driver;
use crate::utils::to_bstr;
use windows::{Win32::Media::Speech::*, Win32::System::Com::*};

pub struct Sapi(Option<ISpeechVoice>, bool);

impl Sapi {
    pub fn new() -> Self {
        unsafe { CoInitialize(None).unwrap() };
        let voice: Option<ISpeechVoice> = unsafe {
            CoCreateInstance(&SpVoice, None, CLSCTX_ALL)
                .map_or_else(|_| None, |x| Some(x))
                .into()
        };
        let active = voice.is_some();
        Sapi(voice, active)
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

    fn speak(&self, text: &str, interrupt: bool) -> bool {
        let bstr = to_bstr(text).unwrap();
        // SVSFlagsAsync does not seem to work
        let mut flags: SpeechVoiceSpeakFlags = SVSFIsNotXML;
        if interrupt {
            flags.0 |= SVSFPurgeBeforeSpeak.0;
        }
        unsafe { self.0.as_ref().unwrap().Speak(&bstr, flags).is_ok() }
    }

    fn is_active(&self) -> bool {
        self.1
    }
}
