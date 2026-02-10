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
