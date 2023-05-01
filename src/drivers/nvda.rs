use libloading::os::windows::{Library, Symbol};
use crate::drivers::Driver;
use std::path::Path;
use widestring::U16CString;


type SpeakText = Symbol<unsafe extern fn(*const u16, bool) -> bool>;
type Braille = Symbol<unsafe extern fn(*const u16) -> bool>;

pub struct NVDA(Library);

impl NVDA {
    pub fn new<P: AsRef<Path>>(library_path: P) -> NVDA {
        let library_path = library_path.as_ref();
        let lib = unsafe { Library::new(library_path).expect("Failed to load library") };
        NVDA(lib)
    }
}

impl Driver for NVDA {
    fn name(&self) -> &'static str {
        "NVDA"
    }
    
    fn is_speaking(&self) -> bool {
        false
    }

    fn speak<S: Into<String>>(&self, text: S, interrupt: bool) -> bool {
        let s = text.into();
        let c_str = U16CString::from_str(s);
        if let Some(s) = c_str.ok() {
            unsafe {
                let speak: SpeakText = self.0.get(b"nvdaController_speakText").unwrap();
                speak(s.as_ptr(), interrupt)
            }
        } else {
            false
        }
    }

    fn braille<S: Into<String>>(&self, text: S) -> bool {
        let s = text.into();
        let c_str = U16CString::from_str(s);
        if let Some(s) = c_str.ok() {
            unsafe {
                let braille: Braille = self.0.get(b"nvdaController_brailleMessage").unwrap();
                braille(s.as_ptr())
            }
        } else {
            false
        }
    }

    fn output(&self, text: &str, interrupt: bool) -> bool {
        self.speak(text, interrupt) || self.braille(text)
    }

    fn silence(&self) -> bool {
        false
    }
}

impl Drop for NVDA {
    fn drop(&mut self) {
        drop(&self.0);
    }
}