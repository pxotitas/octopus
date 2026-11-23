use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::ptr;

const PAM_SUCCESS: c_int = 0;
const PAM_PROMPT_ECHO_OFF: c_int = 1;
const PAM_PROMPT_ECHO_ON: c_int = 2;
const PAM_BUF_ERR: c_int = 5;
const PAM_REFRESH_CRED: c_int = 0x0010;

#[repr(C)]
struct Message {
    style: c_int,
    text: *const c_char,
}

#[repr(C)]
struct Response {
    text: *mut c_char,
    code: c_int,
}

#[repr(C)]
struct Conversation {
    callback: extern "C" fn(c_int, *mut *const Message, *mut *mut Response, *mut c_void) -> c_int,
    data: *mut c_void,
}

#[link(name = "pam")]
extern "C" {
    fn pam_start(
        service: *const c_char,
        user: *const c_char,
        conv: *const Conversation,
        handle: *mut *mut c_void,
    ) -> c_int;
    fn pam_authenticate(handle: *mut c_void, flags: c_int) -> c_int;
    fn pam_acct_mgmt(handle: *mut c_void, flags: c_int) -> c_int;
    fn pam_setcred(handle: *mut c_void, flags: c_int) -> c_int;
    fn pam_end(handle: *mut c_void, status: c_int) -> c_int;
    fn pam_strerror(handle: *mut c_void, code: c_int) -> *const c_char;
}

extern "C" {
    fn calloc(count: usize, size: usize) -> *mut c_void;
    fn strdup(text: *const c_char) -> *mut c_char;
}

/// Answers every password prompt with the typed password. PAM frees what
/// this allocates, so it comes from the C allocator.
extern "C" fn converse(
    count: c_int,
    messages: *mut *const Message,
    responses: *mut *mut Response,
    data: *mut c_void,
) -> c_int {
    if count <= 0 {
        return PAM_BUF_ERR;
    }
    unsafe {
        let replies = calloc(count as usize, std::mem::size_of::<Response>()) as *mut Response;
        if replies.is_null() {
            return PAM_BUF_ERR;
        }
        let password = data as *const c_char;
        for i in 0..count as usize {
            let message = *messages.add(i);
            if message.is_null() {
                continue;
            }
            if matches!((*message).style, PAM_PROMPT_ECHO_OFF | PAM_PROMPT_ECHO_ON) {
                (*replies.add(i)).text = strdup(password);
            }
        }
        *responses = replies;
    }
    PAM_SUCCESS
}

/// Checks `password` for `user` through the PAM `service`. Blocks; run it off
/// the main thread.
pub fn authenticate(service: &str, user: &str, password: &str) -> Result<(), String> {
    let service = CString::new(service).map_err(|e| e.to_string())?;
    let user = CString::new(user).map_err(|e| e.to_string())?;
    let password = CString::new(password).map_err(|_| "senha inválida".to_owned())?;
    let conversation = Conversation {
        callback: converse,
        data: password.as_ptr() as *mut c_void,
    };
    let mut handle: *mut c_void = ptr::null_mut();
    unsafe {
        let mut status = pam_start(service.as_ptr(), user.as_ptr(), &conversation, &mut handle);
        if status == PAM_SUCCESS {
            status = pam_authenticate(handle, 0);
        }
        if status == PAM_SUCCESS {
            status = pam_acct_mgmt(handle, 0);
        }
        if status == PAM_SUCCESS {
            pam_setcred(handle, PAM_REFRESH_CRED);
        }
        let message = if status == PAM_SUCCESS {
            None
        } else {
            let text = pam_strerror(handle, status);
            Some(if text.is_null() {
                "falha na autenticação".to_owned()
            } else {
                CStr::from_ptr(text).to_string_lossy().into_owned()
            })
        };
        if !handle.is_null() {
            pam_end(handle, status);
        }
        message.map_or(Ok(()), Err)
    }
}

/// The first PAM service file present, so the lock works whether or not a
/// screen locker of its own is installed.
pub fn service() -> &'static str {
    ["octopus", "swaylock", "hyprlock"]
        .into_iter()
        .find(|name| std::path::Path::new("/etc/pam.d").join(name).exists())
        .unwrap_or("login")
}
