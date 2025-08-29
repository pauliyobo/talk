//! Jaws driver
#![allow(non_camel_case_types, non_snake_case)]
use crate::drivers::{Command, Driver, TalkError};
use crate::utils::to_bstr;
use crossbeam::channel::{Receiver, Sender, bounded};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;
use windows::Win32::Foundation::{VARIANT_BOOL, VARIANT_FALSE, VARIANT_TRUE};
use windows::Win32::System::Com::*;
use windows::core::{BSTR, HRESULT, interface};
use windows::w;

#[interface("123DEDB4-2CF6-429C-A2AB-CC809E5516CE")]
unsafe trait IJawsApi: IDispatch {
    fn RunScript(&self, ScriptName: BSTR, vbSuccess: *mut VARIANT_BOOL) -> HRESULT;
    fn SayString(
        &self,
        StringToSpeak: BSTR,
        bFlush: VARIANT_BOOL,
        vbSuccess: *mut VARIANT_BOOL,
    ) -> HRESULT;
    fn StopSpeech(&self) -> HRESULT;
    fn Enable(&self, vbNoDDIHooks: VARIANT_BOOL, vbSuccess: *mut VARIANT_BOOL) -> HRESULT;
    fn Disable(&self, vbSuccess: *mut VARIANT_BOOL) -> HRESULT;
    fn RunFunction(&self, FunctionName: BSTR, vbSuccess: *mut VARIANT_BOOL) -> HRESULT;
}

/// loop used in a background thread to listen for messages issuing speak commands
/// This allows us to interact with the COM instance from other threads using crossbeam channels
/// The overhead itself should be relatively minimal
/// # SAFETY
/// `CoInitializeEx()` is going to be entirely managed by this thread, and we expect it to be called at most once per thread
fn jaws_loop(rx: Receiver<Command>, status: oneshot::Sender<Result<(), TalkError>>) {
    let guid = unsafe { CLSIDFromProgID(w!("freedomsci.jawsapi")) };
    if let Err(e) = guid {
        // JAWS is likely not installed on the system or not registered properly
        println!("{:?}", e);
        // send the status to the oneshot receiver so that we may act up on the initialization
        status
            .send(Err(TalkError::DriverInitializationError))
            .unwrap();
        return;
    }
    if let Err(e) = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) } {
        println!("Error in initializing COM: {:?}", e);
        status
            .send(Err(TalkError::DriverInitializationError))
            .unwrap();
        return;
    }
    let jaws: IJawsApi = unsafe { CoCreateInstance(&guid.unwrap(), None, CLSCTX_ALL).unwrap() };
    status.send(Ok(())).unwrap();
    while let Ok(cmd) = rx.recv() {
        match cmd {
            Command::Speak(text, interrupt) => {
                let bstr = to_bstr(&text).unwrap();
                let mut success = VARIANT_FALSE;
                let flush = if interrupt {
                    VARIANT_TRUE
                } else {
                    VARIANT_FALSE
                };
                unsafe { jaws.SayString(bstr, flush, &mut success).is_ok() };
            }
            Command::Shutdown => break,
            Command::IsActive(sender) => sender.send(true).unwrap(),
            _ => println!("Unimplemented"),
        }
    }
    println!("Shutting down COM.");
    unsafe {
        CoUninitialize();
    }
}

#[derive(Debug)]
struct JAWSInner {
    sender: Sender<Command>,
    thread_handle: Option<JoinHandle<()>>,
}

impl Drop for JAWSInner {
    fn drop(&mut self) {
        println!("Killing JAWS Inner");
        if let Err(_) = self.sender.send(Command::Shutdown) {
            // DO nothing, the hcannel may have been already closed because of a previous error
        }
        if let Some(handle) = self.thread_handle.take() {
            handle.join().unwrap();
        }
    }
}

pub struct JAWS {
    inner: Arc<JAWSInner>,
}

impl JAWS {
    pub fn new() -> Result<Self, TalkError> {
        let (sender, receiver) = bounded(1);
        let (osender, oreceiver) = oneshot::channel();
        let thread_handle = std::thread::spawn(move || {
            println!("Starting JAWS loop");
            jaws_loop(receiver, osender);
        });
        let inner = Arc::new(JAWSInner {
            sender,
            thread_handle: Some(thread_handle),
        });
        if let Ok(msg) = oreceiver.recv() {
            match msg {
                Err(e) => return Err(e),
                Ok(()) => {}
            }
        }
        Ok(Self { inner })
    }
}

impl Driver for JAWS {
    fn name(&self) -> &'static str {
        "JAWS"
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
        r.recv_timeout(Duration::from_secs(1)).is_ok()
    }
}
