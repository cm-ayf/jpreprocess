use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{error::JPreprocessErrorKind, JPreprocessError};

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum Keiyoushi {
    /// 自立
    Jiritsu,
    /// 接尾
    Setsubi,
    /// 非自立
    Hijiritsu,
}

impl FromStr for Keiyoushi {
    type Err = JPreprocessError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "自立" => Ok(Self::Jiritsu),
            "接尾" => Ok(Self::Setsubi),
            "非自立" => Ok(Self::Hijiritsu),
            _ => Err(JPreprocessErrorKind::PartOfSpeechParseError
                .with_error(anyhow::anyhow!("Parse failed in Keiyoushi"))),
        }
    }
}

impl Display for Keiyoushi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{},*,*",
            match &self {
                Self::Jiritsu => "自立",
                Self::Setsubi => "接尾",
                Self::Hijiritsu => "非自立",
            },
        )
    }
}