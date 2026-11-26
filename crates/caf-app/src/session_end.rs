//! Clean exit when Windows ends the app's session.
//!
//! Windows sends `WM_ENDSESSION` on shutdown, restart and sign-out, and Restart Manager sends
//! it when an installer or updater needs to replace a running CAMark. tao 0.35 reacts by
//! marking its event loop `Destroyed` while the loop is still pumping, so the next window
//! message panics with "cannot move state from Destroyed" (tao `event_loop/runner.rs`). With
//! `panic = "abort"` that killed the process and showed a crash report on the next launch.
//!
//! We subclass every top-level window of the UI thread (a later subclass runs first) and end
//! the process ourselves on a confirmed end of session, before tao sees the message. Windows
//! is about to terminate the process anyway.

use std::ffi::c_void;

type Hwnd = *mut c_void;
type SubclassProc = unsafe extern "system" fn(Hwnd, u32, usize, isize, usize, usize) -> isize;
type EnumProc = unsafe extern "system" fn(Hwnd, isize) -> i32;

const WM_ENDSESSION: u32 = 0x0016;
/// Any value unique among the subclasses on these windows.
const SUBCLASS_ID: usize = 0x434D_524B; // "CMRK"

#[link(name = "comctl32")]
unsafe extern "system" {
    fn SetWindowSubclass(hwnd: Hwnd, proc: SubclassProc, id: usize, data: usize) -> i32;
    fn DefSubclassProc(hwnd: Hwnd, msg: u32, wparam: usize, lparam: isize) -> isize;
}

#[link(name = "user32")]
unsafe extern "system" {
    fn EnumThreadWindows(thread_id: u32, proc: EnumProc, lparam: isize) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCurrentThreadId() -> u32;
}

unsafe extern "system" fn on_message(
    hwnd: Hwnd,
    msg: u32,
    wparam: usize,
    lparam: isize,
    _id: usize,
    _data: usize,
) -> isize {
    // wParam is FALSE when another application cancelled the shutdown: keep running then.
    if msg == WM_ENDSESSION && wparam != 0 {
        // Deliberate: returning would let tao enter its broken state.
        #[allow(clippy::disallowed_methods)]
        std::process::exit(0);
    }
    // SAFETY: forwards the untouched arguments to the next procedure in the subclass chain.
    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}

unsafe extern "system" fn subclass_window(hwnd: Hwnd, _lparam: isize) -> i32 {
    // SAFETY: `hwnd` belongs to the calling thread, as SetWindowSubclass requires.
    unsafe {
        SetWindowSubclass(hwnd, on_message, SUBCLASS_ID, 0);
    }
    1 // continue enumeration
}

/// Must run on the UI thread, after the main window and the plugins have created theirs
/// (tao's hidden event-target window and the single-instance window included).
pub fn install() {
    // SAFETY: enumerates only the current thread's windows with a matching callback.
    unsafe {
        EnumThreadWindows(GetCurrentThreadId(), subclass_window, 0);
    }
}
