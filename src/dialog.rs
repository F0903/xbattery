use windows::{
    Win32::UI::WindowsAndMessaging::{
        IDYES, MB_ICONERROR, MB_ICONINFORMATION, MB_ICONQUESTION, MB_OK, MB_YESNO,
        MESSAGEBOX_RESULT, MESSAGEBOX_STYLE, MessageBoxW,
    },
    core::HSTRING,
};

pub fn show_info(title: &str, message: &str) {
    show(title, message, MB_OK | MB_ICONINFORMATION);
}

pub fn show_error(title: &str, message: &str) {
    show(title, message, MB_OK | MB_ICONERROR);
}

pub fn ask_yes_no(title: &str, message: &str) -> bool {
    show(title, message, MB_YESNO | MB_ICONQUESTION) == IDYES
}

fn show(title: &str, message: &str, style: MESSAGEBOX_STYLE) -> MESSAGEBOX_RESULT {
    let title = HSTRING::from(title);
    let message = HSTRING::from(message);

    unsafe { MessageBoxW(None, &message, &title, style) }
}
