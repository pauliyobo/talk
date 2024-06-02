//! NVDA driver
use crate::drivers::Driver;
use libloading::os::windows::{Library, Symbol};
use once_cell::sync::OnceCell;
use std::env::current_dir;
use std::path::Path;
use widestring::U16CString;

static NVDA_LIBRARY: OnceCell<Library> = OnceCell::new();
static SPEAK_TEXT: OnceCell<Symbol<unsafe extern "C" fn(*const u16, bool) -> bool>> =
    OnceCell::new();
static BRAILLE_MESSAGE: OnceCell<Symbol<unsafe extern "C" fn(*const u16) -> bool>> =
    OnceCell::new();

pub fn nvda_dll_name() -> &'static str {
    #[cfg(target_arch = "x86")]
    {
        "nvdaControllerClient32.dll"
    }
    #[cfg(not(target_arch = "x86"))]
    {
        "nvdaControllerClient64.dll"
    }
}
fn init_nvda(library_path: Option<&Path>) -> bool {
    NVDA_LIBRARY.get_or_init(|| {
        let path = library_path.map_or_else(
            || {
                let cwd = current_dir().expect("Failed to get current directory");
                cwd.join(nvda_dll_name())
            },
            |p| {
                if p.is_dir() {
                    p.join(nvda_dll_name())
                } else {
                    p.to_path_buf()
                }
            },
        );
        unsafe { Library::new(path).unwrap() }
    });

    let lib = match NVDA_LIBRARY.get() {
        Some(lib) => lib,
        None => return false,
    };

    unsafe {
        SPEAK_TEXT.get_or_init(|| lib.get(b"nvdaController_speakText").unwrap());
        BRAILLE_MESSAGE.get_or_init(|| lib.get(b"nvdaController_brailleMessage").unwrap());
    }

    NVDA_LIBRARY.get().is_some()
}

pub struct NVDA;

impl NVDA {
    pub fn new(library_path: Option<&Path>) -> Self {
        let _ = init_nvda(library_path);
        NVDA
    }
}

impl Driver for NVDA {
    fn name(&self) -> &'static str {
        "NVDA"
    }

    fn speak(&self, text: &str, interrupt: bool) -> bool {
        let c_str = U16CString::from_str(text).unwrap();
        unsafe {
            SPEAK_TEXT
                .get()
                .map_or(false, |speak| speak(c_str.as_ptr(), interrupt))
        }
    }

    fn braille(&self, text: &str) -> bool {
        let c_str = U16CString::from_str(text).unwrap();
        unsafe {
            BRAILLE_MESSAGE
                .get()
                .map_or(false, |braille| braille(c_str.as_ptr()))
        }
    }

    fn is_active(&self) -> bool {
        NVDA_LIBRARY.get().is_some()
    }
}
