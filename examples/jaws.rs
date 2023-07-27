use talk::drivers::{Driver, JAWS};

fn main() {
    let jaws = JAWS::new();
    jaws.output("Speech and braille output with JAWS", false);
}
