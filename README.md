# Talk
This is mainly an experiment to see whether creating something similar to what [tolk](https://github.com/dkager/tolk) was would be worth it.  
The API is expected to break frequently for now, so use at your own risk, if you do.
## How does it work?
Talk makes use of a single concept, `Driver` which is a trait used to implement generic abstractions over screenreaders.
You may find some examples in the [examples](https://github.com/pauliyobo/talk/blob/master/examples) folder
## Example

```rust
use talk::Talk;

fn main() {
    // You can create a driver handle quite easily
    let nvda = Talk::nvda();
    let sapi = Talk::sapi();
    nvda.speak("Speaking from NVDA", false);
    sapi.speak("Speaking from sapi", false);
    // you can cheaply clone the driver handle to use in other threads as well
    // once all references to the handle are dropped
    // the background driver worker will clean up its resources
}
```