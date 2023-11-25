# Talk
This is mainly an experiment to see whether creating something similar to what [tolk](https://github.com/dkager/tolk) was would be worth it.  
The API is expected to break frequently for now, so use at your own risk, if you do.
## How does it work?
Talk makes use of a single concept, `Driver` which is a trait used to implement generic abstractions over screenreaders.
You may find some examples in the [examples](https://github.com/pauliyobo/talk/blob/master/examples) folder
## Example
The API below has been stuctured almost identically as the original tolk's API.

```rust
use talk::Talk;

fn main() {
    let talk = Talk::new();
    // detect the screen reader
    println!("{:?}", talk.detect_screen_reader());
    // output only text
    talk.speak("Text", true);
    // output only braille
    talk.braille("txt");
    // Output both text and braille
    talk.speak("Testing", true);
}
```

In addition though you may also use a single driver if you so desire
```rust

// Will assume the DLL is in the same directory
use talk::drivers::{Driver, NVDA};

fn main() {
    let nvda = NVDA::new("nvdaControllerClient64.dll");
    nvda.speak("This is a test", false);
    nda.braille("Testing braille.");
}
```