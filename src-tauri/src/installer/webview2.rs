use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
use winreg::RegKey;
use windows::core::HSTRING;
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, IDYES, MB_ICONWARNING, MB_YESNO, SW_SHOWNORMAL};

const CLIENT: &str = "{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";
const DOWNLOAD_URL: &str = "https://go.microsoft.com/fwlink/p/?LinkId=2124703";

pub fn valid_pv(pv: &str) -> bool {
    !pv.is_empty() && pv != "0.0.0.0"
}

/// Официальный способ проверки WebView2 Runtime: значение `pv` в EdgeUpdate\Clients.
pub fn installed() -> bool {
    let places = [
        (HKEY_LOCAL_MACHINE, format!(r"SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{CLIENT}")),
        (HKEY_LOCAL_MACHINE, format!(r"SOFTWARE\Microsoft\EdgeUpdate\Clients\{CLIENT}")),
        (HKEY_CURRENT_USER, format!(r"Software\Microsoft\EdgeUpdate\Clients\{CLIENT}")),
    ];
    places.iter().any(|(root, path)| {
        RegKey::predef(*root)
            .open_subkey(path)
            .and_then(|k| k.get_value::<String, _>("pv"))
            .is_ok_and(|pv| valid_pv(&pv))
    })
}

/// Окна ещё нет (без WebView2 его не нарисовать) — спрашиваем системным диалогом.
pub fn prompt_download(lang: &str) {
    let (text, caption) = if lang == "ru" {
        ("Для работы нужен компонент Microsoft WebView2.\n\nОткрыть страницу загрузки?", "Foundry Performance")
    } else {
        ("Microsoft WebView2 Runtime is required.\n\nOpen the download page?", "Foundry Performance")
    };
    unsafe {
        if MessageBoxW(None, &HSTRING::from(text), &HSTRING::from(caption), MB_YESNO | MB_ICONWARNING) == IDYES {
            ShellExecuteW(None, &HSTRING::from("open"), &HSTRING::from(DOWNLOAD_URL), None, None, SW_SHOWNORMAL);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_strings() {
        assert!(!valid_pv(""));
        assert!(!valid_pv("0.0.0.0"));
        assert!(valid_pv("128.0.2739.42"));
    }

    #[test]
    fn this_machine_has_webview2() {
        assert!(installed());
    }
}
