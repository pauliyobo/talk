// Will assume the DLL is in the same directory
use talk::drivers::{Driver, NVDA};

fn main() {
    // let nvda = NVDA::new("nvdaControllerClient64.dll");
    let nvda = NVDA::new(None).unwrap();
    nvda.output("This is a test", false);
}
