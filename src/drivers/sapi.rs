//! Sapi 5.3 Driver
use crate::drivers::Driver;
use windows::{
    core::*,
    Win32::System::Com::*,
    Win32::Media::Speech::*,
};
use widestring::U16CString;

pub struct Sapi(ISpeechVoice);


impl Sapi {
     pub fn new() -> Self {
        unsafe { CoInitialize(None).unwrap() };
        let voice: ISpeechVoice = unsafe { CoCreateInstance(&SpVoice, None, CLSCTX_ALL).expect("Could not initialize Sapi voice.") };
        Sapi(voice)
    }
}

impl Driver for Sapi {
    fn name(&self) -> &'static str {
        "Sapi"
    }

    fn speak<S: Into<String>>(&self, text: S, interrupt: bool) -> bool {
        let text: String = text.into();
        let cstr = U16CString::from_str(&text).unwrap();
        let bstr = BSTR::from_wide(cstr.as_slice()).unwrap();
        // SVSFlagsAsync does not seem to work
        let mut flags: SpeechVoiceSpeakFlags = SVSFIsNotXML;
        if interrupt {
            flags.0 |= SVSFPurgeBeforeSpeak.0;
        }
        unsafe { self.0.Speak(&bstr, flags).is_ok() }
    }
}