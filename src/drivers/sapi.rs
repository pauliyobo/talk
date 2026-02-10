//! Sapi 5.3 Driver
use crate::drivers::Command;
use crate::drivers::Driver;
use crate::utils::to_bstr;
use crossbeam::channel::TryRecvError;
use crossbeam::channel::{Receiver, Sender, bounded};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;
use windows::{
    Win32::Media::Speech::*,
    Win32::System::Com::*,
    Win32::UI::WindowsAndMessaging::{DispatchMessageW, MSG, PM_REMOVE, PeekMessageW},
};

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
    let mut last_stream: i32 = 0;
    loop {
        // We need to manually pump COM messages because
        // using the async flag while speaking would put the speak request in a queue, which wouldn't be processed on this thread
        unsafe {
            let mut msg = MSG::default();
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                DispatchMessageW(&msg);
            }
        };
        match rx.try_recv() {
            Err(TryRecvError::Empty) => std::thread::sleep(Duration::from_millis(1)),
            Err(TryRecvError::Disconnected) => break,
            Ok(cmd) => match cmd {
                Command::Speak(text, interrupt) => {
                    let bstr = to_bstr(&text).unwrap();
                    let mut flags: SpeechVoiceSpeakFlags = SVSFIsNotXML;
                    flags.0 |= SVSFlagsAsync.0;
                    if interrupt {
                        flags.0 |= SVSFPurgeBeforeSpeak.0;
                    }
                    if let Ok(stream) = unsafe { voice.as_ref().unwrap().Speak(&bstr, flags) } {
                        last_stream = stream;
                    }
                }
                Command::IsActive(sender) => sender.send(true).unwrap_or_default(),
                Command::IsSpeaking(sender) => {
                    let status = unsafe {
                        voice
                            .as_ref()
                            .and_then(|v| v.Status().ok())
                            .and_then(|s| s.RunningState().ok())
                            .map(|run_state| run_state == SRSEIsSpeaking)
                            .unwrap()
                    };
                    let _ = sender.send(status);
                }
                Command::Shutdown => break,
                _ => println!("Unimplemented"),
            },
        }
    }
    // After shutdown, keep pumping messages until all queued speech finishes
    if last_stream > 0 {
        if let Some(v) = voice.as_ref() {
            loop {
                unsafe {
                    let mut msg = MSG::default();
                    while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                        DispatchMessageW(&msg);
                    }
                }
                let done = unsafe {
                    v.Status()
                        .ok()
                        .map(|s| {
                            let state = s.RunningState().unwrap_or(SRSEDone);
                            let current = s.CurrentStreamNumber().unwrap_or(0);
                            state == SRSEDone && current >= last_stream
                        })
                        .unwrap_or(true)
                };
                if done {
                    break;
                }
                std::thread::sleep(Duration::from_millis(1));
            }
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
        let _ = self.sender.send(Command::Shutdown);
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
        r.recv().unwrap_or(false)
    }

    fn is_speaking(&self) -> bool {
        let (s, r) = oneshot::channel();
        self.inner.sender.send(Command::IsSpeaking(s)).unwrap();
        r.recv_timeout(Duration::from_secs(5)).unwrap_or(false)
    }
}
