//! Smoke test for the C ABI: no peer needed, just prove the core runs.
//!
//!     cargo run --example wrap_smoke
//!
//! Prints the version it links against, then dials a peer id that cannot
//! exist and waits for the state callback to report the failure.

use std::ffi::{c_char, CStr};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use librustdesk::wrap_ffi::*;

static mut STATES: Option<mpsc::Sender<(i32, String)>> = None;

extern "C" fn on_state(state: i32, message: *const c_char) {
    let text = if message.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(message) }.to_string_lossy().into_owned()
    };
    unsafe {
        if let Some(tx) = (*std::ptr::addr_of!(STATES)).as_ref() {
            let _ = tx.send((state, text));
        }
    }
}

fn main() {
    let (tx, rx) = mpsc::channel();
    unsafe { STATES = Some(tx) };

    wrap_set_callbacks(None, Some(on_state));
    let version = unsafe { CStr::from_ptr(wrap_version()) }.to_string_lossy().into_owned();
    println!("core version: {version}");
    assert!(version.starts_with("rustdesk"), "unexpected version {version:?}");

    // A well-formed but unassigned id: the rendezvous server has nothing to say.
    let id = std::ffi::CString::new("987654321").unwrap();
    let password = std::ffi::CString::new("").unwrap();
    let started = Instant::now();
    let rc = wrap_connect(id.as_ptr(), password.as_ptr());
    println!("wrap_connect -> {rc}");
    assert_eq!(rc, 0, "wrap_connect should start a session thread");

    // Expect at least one state callback (notice or error) and no crash.
    let deadline = Duration::from_secs(45);
    let mut seen = Vec::new();
    while started.elapsed() < deadline {
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok((state, text)) => {
                println!("state {state}: {text}");
                seen.push(state);
                if state == STATE_ERROR || state == STATE_NOTICE {
                    break;
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(e) => panic!("{e}"),
        }
    }
    wrap_disconnect();
    assert!(!seen.is_empty(), "the core never reported back");
    println!("ok: callbacks fired, session thread stayed alive");
}
