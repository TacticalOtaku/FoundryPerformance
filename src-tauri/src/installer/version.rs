#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(pub u32, pub u32, pub u32);

impl Version {
    /// `1.2.3`, `v1.2`, `1.2.3-beta` → Version; предрелизные суффиксы отбрасываются.
    pub fn parse(s: &str) -> Option<Version> {
        let core = s.trim().trim_start_matches('v').split(['-', '+']).next()?;
        let parts: Vec<&str> = core.split('.').collect();
        if parts.len() > 3 {
            return None;
        }
        let n = |i: usize| parts.get(i).map_or(Some(0), |p| p.parse::<u32>().ok());
        Some(Version(n(0)?, n(1)?, n(2)?))
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.0, self.1, self.2)
    }
}

pub fn current() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("valid crate version")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_forms() {
        assert_eq!(Version::parse("0.1.0"), Some(Version(0, 1, 0)));
        assert_eq!(Version::parse("v1.2"), Some(Version(1, 2, 0)));
        assert_eq!(Version::parse("1.2.3-beta"), Some(Version(1, 2, 3)));
    }

    #[test]
    fn rejects_garbage() {
        assert_eq!(Version::parse("a.b"), None);
        assert_eq!(Version::parse("1.2.3.4"), None);
        assert_eq!(Version::parse(""), None);
    }

    #[test]
    fn orders_and_displays() {
        assert!(Version(0, 2, 0) > Version(0, 1, 9));
        assert_eq!(Version(1, 2, 3).to_string(), "1.2.3");
        assert_eq!(current().to_string(), env!("CARGO_PKG_VERSION"));
    }
}
