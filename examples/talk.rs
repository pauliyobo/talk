use talk::Talk;

fn main() {
    let talk = Talk::new();
    println!("{:?}", talk.detect_screen_reader());
    talk.speak("Testing", true);
}
