use crate::model::Locale;

const LANG_RUSSIAN: u16 = 0x19;

pub fn effective(l: Locale) -> &'static str {
    match l {
        Locale::Ru => "ru",
        Locale::En => "en",
        Locale::Auto => {
            let lang = unsafe { windows::Win32::Globalization::GetUserDefaultUILanguage() };
            if lang & 0x3ff == LANG_RUSSIAN {
                "ru"
            } else {
                "en"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_locales() {
        assert_eq!(effective(Locale::Ru), "ru");
        assert_eq!(effective(Locale::En), "en");
        assert!(["ru", "en"].contains(&effective(Locale::Auto)));
    }
}
