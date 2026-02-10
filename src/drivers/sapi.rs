//! Sapi 5.3 Driver
use crate::drivers::Command;
use crate::drivers::Driver;
use crate::utils::to_bstr;
use crossbeam::channel::{Receiver, Sender, bounded};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;
use windows::{Win32::Media::Speech::*, Win32::System::Com::*};

/// loop used in a background thread to listen for messages issuing speak commands
/// This allows us to interact with the COM instance from other threads using crossbeam channels
/// The overhead itself should be relatively minimal
/// # Safety
/// `CoInitializeEx()` is going to be entirely managed by this thread, and we expect it to be called at most once per thread
fn sapi_loop(rx: Receiver<Command>) {
    unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED).unwrap() };
    let voice: Option<ISpeechVoice> = unsafe {
        CoCreateInstance(&SpVoice, None, CLSCTX_ALL)
            .map_or_else(|_| None, |x| Some(x))
            .into()
    };
    rx.iter().for_each(|cmd| {
        println!("Received {:?}", cmd);
        match cmd {
            Command::Speak(text, interrupt) => {
                let bstr = to_bstr(&text).unwrap();
                // SVSFlagsAsync does not seem to work
                let mut flags: SpeechVoiceSpeakFlags = SVSFIsNotXML;
                flags.0 |= SVSFlagsAsync.0;
                if interrupt {
                    flags.0 |= SVSFPurgeBeforeSpeak.0;
                }
                let _ = unsafe { voice.as_ref().unwrap().Speak(&bstr, flags).is_ok() };
            }
            Command::IsActive(sender) => sender.send(true).unwrap_or_default(),
            Command::IsSpeaking(sender) => {
                let status = unsafe {
                    voice
                        .as_ref()
                        .and_then(|v| v.Status().ok())
                        .and_then(|s| s.RunningState().ok())
                        .map(|run_state| run_state == SRSEIsSpeaking)
                        .unwrap_or(false)
                };
                if !sender.is_closed() {
                    sender.send(status).unwrap();
                } else {
                    println!("sender is closed.");
                }
            }
            _ => println!("Unimplemented"),
        }
    });
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
        // self.sender.send(Command::Shutdown).unwrap();        
        println!("{:?}", self.sender.capacity());
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

impl Drop for Sapi {
    fn drop(&mut self) {
        loop {
            if !self.is_speaking() {
                break;
            }
        }
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
        r.recv().unwrap_or(false)
    }

    fn is_speaking(&self) -> bool {
        let (s, r) = oneshot::channel();
        self.inner.sender.send(Command::IsSpeaking(s)).unwrap();
        r.recv_timeout(Duration::from_secs(5)).unwrap_or(false)
    }
}
