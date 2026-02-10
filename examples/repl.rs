use std::io;
use talk::Talk;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut line = String::new();
    let mut talk = Talk::sapi();
    talk.speak("Welcome to this uninspiring TTS REPL", false);
    loop {
        println!("Type a message");
        io::stdin().read_line(&mut line)?;
        match line.to_lowercase().as_str() {
            "nvda\r\n" => {
                talk.speak("switching to NVDA", false);
                talk = Talk::nvda();
                talk.speak("Switched to NVDA", false);

            }
            "exit\r\n" => break,
            "speaking\r\n" => println!("{}", talk.is_speaking()),
            msg => {
                talk.speak(msg, true);
            }
        };
        line.clear();
    }
    Ok(())
}
