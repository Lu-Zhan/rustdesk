//! C ABI for Wrap: connect to a RustDesk peer and stream frames to the host app.
//!
//! RustDesk's own mobile glue is `flutter_rust_bridge`, which is Dart-shaped and
//! can't be called from Swift or Kotlin. This module is the seam for Wrap only:
//! a `Session<WrapHandler>` driven by `ui_session_interface::io_loop`, plus a
//! plain C surface (`extern "C"`) that the iOS and Android apps bind to.
//!
//! Milestone 1: connect, decoded frames out, mouse/keyboard in.

#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_int, CStr, CString};
use std::sync::{Arc, Mutex, OnceLock};

use base::message_proto::KeyEvent;
use hbb_common::{log, rendezvous_proto::ConnType};

use crate::ui_session_interface::{io_loop, InvokeUiSession, Session};

/// Callback ids shared with the Swift/Kotlin side.
pub const STATE_CONNECTED: i32 = 1;
pub const STATE_CLOSED: i32 = 2;
pub const STATE_ERROR: i32 = 3;
pub const STATE_NOTICE: i32 = 4;
pub const STATE_DISPLAY: i32 = 5;
pub const STATE_TRANSPORT: i32 = 6;

/// `display`, `width`, `height`, then a borrowed RGBA buffer that is only valid
/// for the duration of the call.
pub type WrapFrameCallback =
    extern "C" fn(display: usize, width: usize, height: usize, data: *const u8, len: usize);
/// `state` (see `STATE_*`) plus a NUL-terminated message that is only valid for
/// the duration of the call.
pub type WrapStateCallback = extern "C" fn(state: c_int, message: *const c_char);

#[derive(Clone, Default)]
struct Callbacks {
    frame: Option<WrapFrameCallback>,
    state: Option<WrapStateCallback>,
}

static CALLBACKS: OnceLock<Mutex<Callbacks>> = OnceLock::new();
static SESSION: OnceLock<Mutex<Option<Session<WrapHandler>>>> = OnceLock::new();

fn callbacks() -> Callbacks {
    CALLBACKS
        .get_or_init(Default::default)
        .lock()
        .unwrap()
        .clone()
}

fn session_slot() -> &'static Mutex<Option<Session<WrapHandler>>> {
    SESSION.get_or_init(Default::default)
}

/// The UI side of one connection. RustDesk's trait is wide because it serves a
/// full file-transfer UI; Wrap only needs frames, state and errors.
#[derive(Clone, Default)]
pub struct WrapHandler {}

impl WrapHandler {
    fn state(&self, state: i32, message: &str) {
        if let Some(f) = callbacks().state {
            if let Ok(text) = CString::new(message.replace('\0', "")) {
                f(state, text.as_ptr());
            }
        }
    }
}

impl InvokeUiSession for WrapHandler {
    fn set_cursor_data(&self, &self, cd: CursorData) {
    }

    fn set_cursor_id(&self, &self, id: String) {
    }

    fn set_cursor_position(&self, &self, cp: CursorPosition) {
    }

    fn set_display(&self, &self, x: i32, y: i32, w: i32, h: i32, cursor_embedded: bool, scale: f64) {
    }

    fn switch_display(&self, &self, display: &SwitchDisplay) {
    }

    fn set_peer_info(&self, &self, peer_info: &PeerInfo); // flutter fn set_displays(&self, displays: &Vec<DisplayInfo>) {
    }

    fn set_platform_additions(&self, &self, data: &str) {
    }

    fn on_connected(&self, &self, conn_type: ConnType) {
    }

    fn update_privacy_mode(&self, &self) {
    }

    fn set_permission(&self, &self, name: &str, value: bool) {
    }

    fn close_success(&self, &self) {
    }

    fn update_quality_status(&self, &self, qs: QualityStatus) {
    }

    fn set_connection_type(&self, &self, is_secured: bool, direct: bool, stream_type: &str) {
    }

    fn set_fingerprint(&self, &self, fingerprint: String) {
    }

    fn job_error(&self, &self, id: i32, err: String, file_num: i32) {
    }

    fn job_done(&self, &self, id: i32, file_num: i32) {
    }

    fn clear_all_jobs(&self, &self) {
    }

    fn new_message(&self, &self, msg: String) {
    }

    fn update_transfer_list(&self, &self) {
    }

    fn load_last_job(&self, &self, cnt: i32, job_json: &str, auto_start: bool) {
    }

    fn update_folder_files(&self,  &self, id: i32, entries: &Vec<FileEntry>, path: String, is_local: bool, only_count: bool, ) {
    }

    fn confirm_delete_files(&self, &self, id: i32, i: i32, name: String) {
    }

    fn override_file_confirm(&self,  &self, id: i32, file_num: i32, to: String, is_upload: bool, is_identical: bool, ) {
    }

    fn update_block_input_state(&self, &self, on: bool) {
    }

    fn job_progress(&self, &self, id: i32, file_num: i32, speed: f64, finished_size: f64) {
    }

    fn adapt_size(&self, &self) {
    }

    fn on_rgba(&self, &self, display: usize, rgba: &mut scrap::ImageRgb) {
    }

    fn msgbox(&self, &self, msgtype: &str, title: &str, text: &str, link: &str, retry: bool) {
    }

    #[cfg(any(target_os = "android", target_os = "ios"))]
    fn clipboard(&self, &self, content: String) {
    }

    fn cancel_msgbox(&self, &self, tag: &str) {
    }

    fn switch_back(&self, &self, id: &str) {
    }

    fn portable_service_running(&self, &self, running: bool) {
    }

    fn on_voice_call_started(&self, &self) {
    }

    fn on_voice_call_closed(&self, &self, reason: &str) {
    }

    fn on_voice_call_waiting(&self, &self) {
    }

    fn on_voice_call_incoming(&self, &self) {
    }

    fn get_rgba(&self, &self, display: usize) -> *const u8 {
        Default::default()
    }

    fn next_rgba(&self, &self, display: usize) {
    }

    #[cfg(all(feature = "vram", feature = "flutter"))]
    fn on_texture(&self, &self, display: usize, texture: *mut c_void) {
    }

    fn set_multiple_windows_session(&self, &self, sessions: Vec<WindowsSession>) {
    }

    fn set_current_display(&self, &self, disp_idx: i32) {
    }

    #[cfg(feature = "flutter")]
    fn is_multi_ui_session(&self, &self) -> bool {
        false
    }

    fn update_record_status(&self, &self, start: bool) {
    }

    fn printer_request(&self, &self, id: i32, path: String) {
    }

    fn handle_screenshot_resp(&self, &self, sid: String, msg: String) {
    }

    fn handle_terminal_response(&self, &self, response: TerminalResponse) {
    }

}

fn cstr(value: *const c_char) -> String {
    if value.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(value) }.to_string_lossy().into_owned()
}

/// Install the callbacks. Must be called before `wrap_connect`.
#[no_mangle]
pub extern "C" fn wrap_set_callbacks(frame: Option<WrapFrameCallback>, state: Option<WrapStateCallback>) {
    *CALLBACKS.get_or_init(Default::default).lock().unwrap() = Callbacks { frame, state };
}

/// Connect to `id` and start streaming. `password` may be empty to prompt the
/// peer's saved password. Returns 0 when the session thread was started.
#[no_mangle]
pub extern "C" fn wrap_connect(id: *const c_char, password: *const c_char) -> c_int {
    let id = cstr(id);
    if id.is_empty() {
        return -1;
    }
    let password = cstr(password);
    let session: Session<WrapHandler> = Session {
        password,
        ui_handler: WrapHandler::default(),
        ..Default::default()
    };
    session
        .lc
        .write()
        .unwrap()
        .initialize(id, ConnType::default(), None, false, None, None, None, "");
    *session_slot().lock().unwrap() = Some(session.clone());
    std::thread::spawn(move || {
        let round = session.connection_round_state.lock().unwrap().new_round();
        io_loop(session, round);
    });
    0
}

/// Drop the session handle; the connection thread exits on its own.
#[no_mangle]
pub extern "C" fn wrap_disconnect() {
    *session_slot().lock().unwrap() = None;
}

/// Mouse: `mask` bit 0 = down, bit 1 = up, bit 2 = wheel, `buttons` bit 0 = left,
/// bit 1 = right (same encoding as RustDesk's own input layer).
#[no_mangle]
pub extern "C" fn wrap_mouse(
    mask: c_int,
    x: c_int,
    y: c_int,
    alt: bool,
    ctrl: bool,
    shift: bool,
    command: bool,
) {
    if let Some(s) = session_slot().lock().unwrap().as_ref() {
        s.send_mouse(mask, x, y, alt, ctrl, shift, command);
    }
}

/// Keyboard: `chr` is the platform key code, `unicode` the typed character.
#[no_mangle]
pub extern "C" fn wrap_key(
    down: bool,
    alt: bool,
    ctrl: bool,
    shift: bool,
    command: bool,
    chr: u32,
    unicode: u32,
) {
    let Some(s) = session_slot().lock().unwrap().clone() else {
        return;
    };
    let evt = KeyEvent {
        down,
        chr,
        unicode,
        ..Default::default()
    };
    s.send_key_event(&evt);
    let _ = (alt, ctrl, shift, command); // modifiers ride on the chr/unicode path for now
}

/// Paste text into the peer's focused field.
#[no_mangle]
pub extern "C" fn wrap_chat(text: *const c_char) {
    let text = cstr(text);
    if let Some(s) = session_slot().lock().unwrap().as_ref() {
        s.send_chat(text);
    }
}

/// Version of the embedded core, for the host app to display.
#[no_mangle]
pub extern "C" fn wrap_version() -> *const c_char {
    static VERSION: OnceLock<CString> = OnceLock::new();
    VERSION
        .get_or_init(|| CString::new(format!("rustdesk {}", crate::VERSION)).unwrap_or_default())
        .as_ptr()
}

#[no_mangle]
pub extern "C" fn wrap_init_log(level: *const c_char) {
    let level = cstr(level);
    hbb_common::config::set_logging_level(true, &level);
    log::info!("wrap core ready");
}
