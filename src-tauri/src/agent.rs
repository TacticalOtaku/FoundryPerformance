use crate::model::{Levers, ProfileId};
use serde::Serialize;
use std::collections::BTreeMap;

pub const AGENT_JS: &str = include_str!("../../agent/dist/agent.js");

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Boot<'a> {
    pub server_id: &'a str,
    pub profile_id: ProfileId,
    pub presets: &'a BTreeMap<ProfileId, Levers>,
    pub locale: &'a str,
    pub measure_only: bool,
}

pub fn script(boot: &Boot) -> String {
    let json = serde_json::to_string(boot).expect("Boot is always serializable");
    format!("window.__FP_BOOT__ = {json};\n{AGENT_JS}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile;

    #[test]
    fn script_embeds_boot_before_agent() {
        let presets = profile::presets();
        let s = script(&Boot { server_id: "a1", profile_id: ProfileId::Balance, presets: &presets, locale: "ru", measure_only: false });
        assert!(s.starts_with("window.__FP_BOOT__ = {"));
        assert!(s.contains("\"profileId\":\"balance\""));
        assert!(s.contains("\"measureOnly\":false"));
        assert!(s.contains("\"potato\":{"));
        assert!(s.ends_with(AGENT_JS));
    }
}
