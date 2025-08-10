use std::thread;
use talk::drivers::{Driver, Sapi};

fn main() {
    let sapi = Sapi::new();
    sapi.speak("test", true);
    sapi.speak("second test", true);
    for i in 0..5 {
        let s = sapi.clone();
        let _ = thread::spawn(move || {
            s.speak(&format!("speaking from thread {}", i + 1), false);
        })
        .join()
        .unwrap();
    }
    println!("{}", sapi.is_active());
    sapi.silence();
    println!("{}", sapi.is_active());
}
