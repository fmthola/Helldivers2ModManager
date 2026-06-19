use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", tag = "For")]
pub enum Config {
    #[serde(rename_all = "PascalCase")]
    Legacy {
        guid: Uuid,
        enabled: bool,
        selected: usize,
    },
    #[serde(rename_all = "PascalCase")]
    V1 {
        guid: Uuid,
        enabled: bool,
        toggled: Vec<bool>,
        selected: Vec<usize>,
    },
    #[serde(rename_all = "PascalCase")]
    V2 {
        guid: Uuid,
        enabled: bool,
        toggled: Vec<bool>,
        selected: Vec<usize>,
    }
}

impl Config {
    pub fn uuid(&self) -> &Uuid {
        match self {
            Config::Legacy { guid, .. } => guid,
            Config::V1 { guid, .. } => guid,
            Config::V2 { guid, .. } => guid,
        }
    }

    pub fn enabled(&self) -> bool {
        match self {
            Config::Legacy { enabled, .. } => *enabled,
            Config::V1 { enabled, .. } => *enabled,
            Config::V2 { enabled, .. } => *enabled,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", tag = "Version")]
pub enum Profile {
    #[serde(rename_all = "PascalCase")]
    V1 {
        name: String,
        configs: Vec<Config>,
    }
}

impl Profile {
    pub fn new(name: &str) -> Self {
        Profile::V1 {
            name: name.to_string(),
            configs: Vec::new()
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Profile::V1 { name, .. } => name,
        }
    }
    
    pub fn configs(&self) -> &[Config] {
        match self {
            Profile::V1 { configs, .. } => configs.as_slice(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ProfilesConfig {
    pub profiles: Vec<Profile>,
    pub active: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_exposes_uuid_and_enabled() {
        let id = Uuid::from_u128(1);
        let legacy = Config::Legacy { guid: id, enabled: false, selected: 0 };
        assert_eq!(legacy.uuid(), &id);
        assert!(!legacy.enabled());

        let v1 = Config::V1 { guid: id, enabled: true, toggled: vec![true], selected: vec![0] };
        assert_eq!(v1.uuid(), &id);
        assert!(v1.enabled());

        let v2 = Config::V2 { guid: id, enabled: true, toggled: vec![], selected: vec![] };
        assert!(v2.enabled());
    }

    #[test]
    fn profile_accessors() {
        let p = Profile::new("Default");
        assert_eq!(p.name(), "Default");
        assert!(p.configs().is_empty());
    }
}