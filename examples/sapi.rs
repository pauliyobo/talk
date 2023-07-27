use talk::drivers::{Driver, Sapi};

fn main() {
    let s = Sapi::new();
    s.speak("Test", false);
    s.speak("another long test you should hopefully find right", true);
}
