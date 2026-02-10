use talk::Talk;

fn main() {
    let talk = Talk::nvda();
    talk.output("Testing", true);
}
