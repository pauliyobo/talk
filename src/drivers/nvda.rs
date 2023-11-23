//! NVDA driver
use crate::drivers::Driver;
use anyhow::bail;
use libloading::os::windows::{Library, Symbol};
use std::env::current_dir;
use std::path::Path;
use widestring::U16CString;

/// Detects the name of the right NVDA DLL to call
pub fn nvda_dll_name() -> &'static str {
    #[cfg(target_arch = "x86")]
    {
        return "nvdaControllerClient32.dll";
    }
    "nvdaControllerClient64.dll"
}

type SpeakText = Symbol<unsafe extern "C" fn(*const u16, bool) -> bool>;
type Braille = Symbol<unsafe extern "C" fn(*const u16) -> bool>;

pub struct NVDA(Library);

impl NVDA {
    /// Createa NVDA driver
    /// The driver expects a library path to load the controller DLL from
    /// If the library_path is None, the library will be searched in the current directory
    pub fn new(library_path: Option<&Path>) -> anyhow::Result<NVDA> {
        let cwd = current_dir()?;
        let library_path = match library_path {
            Some(p) => p,
            None => cwd.as_path(),
        };
        let library_path = library_path.join(nvda_dll_name());
        if !library_path.exists() {
            bail!(format!("Could not load {}", library_path.display()))
        }
        let lib = unsafe { Library::new(library_path)? };
        Ok(NVDA(lib))
    }
}

impl Driver for NVDA {
    fn name(&self) -> &'static str {
        "NVDA"
    }

    fn speak(&self, text: &str, interrupt: bool) -> bool {
        let c_str = U16CString::from_str(text).unwrap();
        unsafe {
            let speak: SpeakText = self.0.get(b"nvdaController_speakText").unwrap();
            speak(c_str.as_ptr(), interrupt)
        }
    }

    fn braille(&self, text: &str) -> bool {
        let c_str = U16CString::from_str(text).unwrap();
        unsafe {
            let braille: Braille = self.0.get(b"nvdaController_brailleMessage").unwrap();
            braille(c_str.as_ptr())
        }
    }
}
