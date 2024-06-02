//! NVDA driver
use std::path::PathBuf;
use talk::drivers::{Driver, NVDA};
fn main() {
    let nvda_dll_path = PathBuf::from("nvdaControllerClient64.dll");
    let nvda = NVDA::new(Some(nvda_dll_path.as_path()));

    nvda.speak("Hello, NVDA!", true);
    nvda.braille("Hello, NVDA braille!");
}
