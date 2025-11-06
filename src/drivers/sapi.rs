//! Sapi 5.3 Driver
use crate::drivers::Command;
use crate::drivers::Driver;
use crate::utils::to_bstr;
use crossbeam::channel::{Receiver, Sender, bounded};
use std::sync::Arc;
use std::thread::JoinHandle;
use windows::{Win32::Media::Speech::*, Win32::System::Com::*};

/// loop used in a background thread to listen for messages issuing speak commands
/// This allows us to interact with the COM instance from other threads using crossbeam channels
/// The overhead itself should be relatively minimal
/// # SAFETY
/// `CoInitializeEx()` is going to be entirely managed by this thread, and we expect it to be called at most once per thread
fn sapi_loop(rx: Receiver<Command>) {
    unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED).unwrap() };
    let voice: Option<ISpeechVoice> = unsafe {
        CoCreateInstance(&SpVoice, None, CLSCTX_ALL)
            .map_or_else(|_| None, |x| Some(x))
            .into()
    };
    while let Ok(cmd) = rx.recv() {
        match cmd {
            Command::Speak(text, interrupt) => {
                let bstr = to_bstr(&text).unwrap();
                // SVSFlagsAsync does not seem to work
                let mut flags: SpeechVoiceSpeakFlags = SVSFIsNotXML;
                if interrupt {
                    flags.0 |= SVSFPurgeBeforeSpeak.0;
                }
                let _ = unsafe { voice.as_ref().unwrap().Speak(&bstr, flags).is_ok() };
            }
            Command::Shutdown => break,
            Command::IsActive(sender) => sender.send(true).unwrap_or_default(),
            _ => println!("Unimplemented"),
        }
    }
    unsafe {
        CoUninitialize();
    }
}

#[derive(Debug)]
struct SapiInner {
    sender: Sender<Command>,
    thread_handle: Option<JoinHandle<()>>,
}

impl Drop for SapiInner {
    fn drop(&mut self) {
        self.sender.send(Command::Shutdown).unwrap();
        if let Some(handle) = self.thread_handle.take() {
            handle.join().unwrap();
        }
    }
}

#[derive(Clone, Debug)]
pub struct Sapi {
    inner: Arc<SapiInner>,
}

impl Sapi {
    pub fn new() -> Self {
        let (sender, r) = bounded(1);
        let thread_handle = std::thread::spawn(move || {
            sapi_loop(r);
        });
        let inner = Arc::new(SapiInner {
            sender,
            thread_handle: Some(thread_handle),
        });
        Self { inner }
    }
}

impl Default for Sapi {
    fn default() -> Self {
        Self::new()
    }
}

impl Driver for Sapi {
    fn name(&self) -> &'static str {
        "Sapi"
    }

    fn speak(&self, text: &str, interrupt: bool) -> bool {
        self.inner
            .sender
            .send(Command::Speak(text.into(), interrupt))
            .unwrap();
        true
    }

    fn is_active(&self) -> bool {
        let (s, r) = oneshot::channel();
        self.inner.sender.send(Command::IsActive(s)).unwrap();
        r.recv().unwrap()
    }
}
