use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct SerifSection {
    #[serde(rename = "type")]
    pub r#type: SerifType,
    #[serde(rename = "speaker")]
    pub speaker: String,
    #[serde(rename = "text")]
    pub text: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Default)]
pub enum SerifType {
    #[default]
    #[serde(rename = "Serif")]
    Serif,
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct AudioSection {
    #[serde(rename = "type")]
    pub r#type: AudioType,
    #[serde(rename = "url")]
    pub url: String,
    #[serde(rename = "from", skip_serializing_if = "Option::is_none")]
    pub from: Option<f64>,
    #[serde(rename = "to", skip_serializing_if = "Option::is_none")]
    pub to: Option<f64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Default)]
pub enum AudioType {
    #[default]
    #[serde(rename = "Audio")]
    Audio,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Section {
    SerifSection(Box<SerifSection>),
    AudioSection(Box<AudioSection>),
}

impl Default for Section {
    fn default() -> Self {
        Self::SerifSection(Default::default())
    }
}
