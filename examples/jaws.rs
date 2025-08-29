use talk::drivers::{Driver, JAWS};

fn main() {
    let jaws = JAWS::new().unwrap();
    jaws.output("Speech and braille output with JAWS", false);
}
