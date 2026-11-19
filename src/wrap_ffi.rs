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
use base::message_proto::*;
use hbb_common::{log, rendezvous_proto::ConnType};

use crate::client::QualityStatus;
use crate::ui_session_interface::*;
use crate::ui_session_interface::{io_loop, InvokeUiSession, Session};
use base::message_proto::key_event;

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
    fn set_cursor_data(&self, cd: CursorData) {}

    fn set_cursor_id(&self, id: String) {}

    fn set_cursor_position(&self, cp: CursorPosition) {}

    fn set_display(&self, x: i32, y: i32, w: i32, h: i32, cursor_embedded: bool, scale: f64) {
        self.state(
            STATE_DISPLAY,
            &format!("{w}x{h} at {x},{y} scale {scale} cursor {cursor_embedded}"),
        );
    }

    fn switch_display(&self, display: &SwitchDisplay) {}

    fn set_peer_info(&self, peer_info: &PeerInfo) {}

    fn set_displays(&self, displays: &Vec<DisplayInfo>) {}

    fn set_platform_additions(&self, data: &str) {}

    fn on_connected(&self, conn_type: ConnType) {
        self.state(STATE_CONNECTED, "");
    }

    fn update_privacy_mode(&self) {}

    fn set_permission(&self, name: &str, value: bool) {}

    fn close_success(&self) {
        self.state(STATE_CLOSED, "");
    }

    fn update_quality_status(&self, qs: QualityStatus) {}

    fn set_connection_type(&self, is_secured: bool, direct: bool, stream_type: &str) {
        let kind = if direct { "direct" } else { "relay" };
        let secured = if is_secured { "secured" } else { "unsecured" };
        self.state(STATE_TRANSPORT, &format!("{kind} {secured} {stream_type}"));
    }

    fn set_fingerprint(&self, fingerprint: String) {}

    fn job_error(&self, id: i32, err: String, file_num: i32) {}

    fn job_done(&self, id: i32, file_num: i32) {}

    fn clear_all_jobs(&self) {}

    fn new_message(&self, msg: String) {}

    fn update_transfer_list(&self) {}

    fn load_last_job(&self, cnt: i32, job_json: &str, auto_start: bool) {}

    fn update_folder_files(
        &self,
        id: i32,
        entries: &Vec<FileEntry>,
        path: String,
        is_local: bool,
        only_count: bool,
    ) {
    }

    fn confirm_delete_files(&self, id: i32, i: i32, name: String) {}

    fn override_file_confirm(
        &self,
        id: i32,
        file_num: i32,
        to: String,
        is_upload: bool,
        is_identical: bool,
    ) {
    }

    fn update_block_input_state(&self, on: bool) {}

    fn job_progress(&self, id: i32, file_num: i32, speed: f64, finished_size: f64) {}

    fn adapt_size(&self) {}

    fn on_rgba(&self, display: usize, rgba: &mut scrap::ImageRgb) {
        // Decoded frames arrive as RGBA: hand the bytes straight to the host app.
        if let Some(f) = callbacks().frame {
            f(display, rgba.w, rgba.h, rgba.raw.as_ptr(), rgba.raw.len());
        }
    }

    fn msgbox(&self, msgtype: &str, title: &str, text: &str, link: &str, retry: bool) {
        let level = if msgtype == "error" {
            STATE_ERROR
        } else {
            STATE_NOTICE
        };
        self.state(
            level,
            &format!("{title}: {text}{}", if retry { " (retry)" } else { "" }),
        );
    }

    #[cfg(any(target_os = "android", target_os = "ios"))]
    fn clipboard(&self, content: String) {}

    fn cancel_msgbox(&self, tag: &str) {}

    fn switch_back(&self, id: &str) {}

    fn portable_service_running(&self, running: bool) {}

    fn on_voice_call_started(&self) {}

    fn on_voice_call_closed(&self, reason: &str) {}

    fn on_voice_call_waiting(&self) {}

    fn on_voice_call_incoming(&self) {}

    fn get_rgba(&self, display: usize) -> *const u8 {
        std::ptr::null()
    }

    fn next_rgba(&self, display: usize) {}

    #[cfg(all(feature = "vram", feature = "flutter"))]
    fn on_texture(&self, display: usize, texture: *mut c_void) {}

    fn set_multiple_windows_session(&self, sessions: Vec<WindowsSession>) {}

    fn set_current_display(&self, disp_idx: i32) {}

    #[cfg(feature = "flutter")]
    fn is_multi_ui_session(&self) -> bool {
        false
    }

    fn update_record_status(&self, start: bool) {}

    fn printer_request(&self, id: i32, path: String) {}

    fn handle_screenshot_resp(&self, sid: String, msg: String) {}

    fn handle_terminal_response(&self, response: TerminalResponse) {}
}

fn cstr(value: *const c_char) -> String {
    if value.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(value) }
        .to_string_lossy()
        .into_owned()
}

/// Install the callbacks. Must be called before `wrap_connect`.
#[no_mangle]
pub extern "C" fn wrap_set_callbacks(
    frame: Option<WrapFrameCallback>,
    state: Option<WrapStateCallback>,
) {
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
    session.lc.write().unwrap().initialize(
        id,
        ConnType::default(),
        None,
        false,
        None,
        None,
        None,
        "",
    );
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
    // `chr` is a position code, `unicode` a typed character: the proto keeps them
    // in a oneof, so pick the arm the caller filled in.
    let evt = KeyEvent {
        down,
        union: Some(if unicode != 0 {
            key_event::Union::Unicode(unicode)
        } else {
            key_event::Union::Chr(chr)
        }),
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
    let _ = level; // hbb_common's logger is initialised by the host app
    log::info!("wrap core ready");
}
