use std::thread;
use std::time::Duration;
use talk::drivers::{Driver, Sapi};

fn main() {
    let sapi = Sapi::new();
    sapi.speak("test", false);
    sapi.speak("second test", false);
    /* for i in 0..3 {
        let s = sapi.clone();
        let _ = thread::spawn(move || {
            s.speak(&format!("speaking from thread {}", i + 1), false);
        })
        .join()
        .unwrap();
    } */
    println!("{}", sapi.is_active());
    sapi.speak("test", false);
    sapi.speak("other test", false);
    sapi.speak(
        "Incredibly long test that should hopefully trigger the bug",
        false,
    );
    println!("{:?}", sapi.is_speaking());
    // drop(sapi);
    std::thread::sleep(Duration::from_secs(3));
}
