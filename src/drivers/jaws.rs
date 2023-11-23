//! Jaws driver
#![allow(non_camel_case_types, non_snake_case)]
use crate::drivers::Driver;
use crate::utils::to_bstr;
use windows::core::{interface, BSTR, HRESULT};
use windows::w;
use windows::Win32::Foundation::{VARIANT_BOOL, VARIANT_FALSE, VARIANT_TRUE};
use windows::Win32::System::Com::*;

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

pub struct JAWS(IJawsApi);

impl JAWS {
    pub fn new() -> Self {
        let guid = unsafe { CLSIDFromProgID(w!("freedomsci.jawsapi")).unwrap() };
        unsafe { CoInitializeEx(None, COINIT_MULTITHREADED).unwrap() };
        let jaws: IJawsApi = unsafe {
            CoCreateInstance(&guid, None, CLSCTX_ALL).expect("Failed to initialize JAWS API")
        };
        JAWS(jaws)
    }
}

impl Default for JAWS {
    fn default() -> Self {
        Self::new()
    }
}

impl Driver for JAWS {
    fn name(&self) -> &'static str {
        "JAWS"
    }

    fn speak(&self, text: &str, interrupt: bool) -> bool {
        let bstr = to_bstr(text).unwrap();
        let mut success = VARIANT_FALSE;
        let flush = if interrupt {
            VARIANT_TRUE
        } else {
            VARIANT_FALSE
        };
        unsafe { self.0.SayString(bstr, flush, &mut success).is_ok() && success == VARIANT_TRUE }
    }

    fn braille(&self, text: &str) -> bool {
        // To output braille with JAWS we need to run the script BrailleString("text")
        let text = to_bstr(&format!("BrailleString(\"{}\")", text)).unwrap();
        let mut success = VARIANT_FALSE;
        unsafe { self.0.RunScript(text, &mut success).is_ok() && success == VARIANT_TRUE }
    }

    fn silence(&self) {
        unsafe { self.0.StopSpeech().unwrap() }
    }
}
