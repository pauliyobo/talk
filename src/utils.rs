use anyhow::Result;
use widestring::U16CString;
use windows::core::BSTR;

/// Convert a str  to BSTR
pub fn to_bstr(text: &str) -> Result<BSTR> {
    let bstr = BSTR::from_wide(U16CString::from_str(text)?.as_slice())?;
    Ok(bstr)
}
