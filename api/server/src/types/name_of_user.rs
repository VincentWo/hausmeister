use std::{ops::Deref, str::FromStr};

use color_eyre::{eyre::eyre, Report};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(try_from = "&str")]
pub(crate) struct NameOfUser(String);

impl FromStr for NameOfUser {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            Err(eyre!("Name can't be empty"))
        } else if s.len() > 265 {
            Err(eyre!("{s:?} can't be longer than 265 bytes/characters"))
        } else if s.chars().any(|c| c.is_control()) {
            Err(eyre!(
                "{s:?} can't contain control chars/unicode scalar values"
            ))
        } else {
            Ok(Self(s.into()))
        }
    }
}

impl TryFrom<&str> for NameOfUser {
    type Error = Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl Deref for NameOfUser {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for NameOfUser {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::NameOfUser;
    use proptest::proptest;

    #[test]
    fn empty_name_is_rejected() {
        let _ = "".parse::<NameOfUser>().unwrap_err();
    }

    proptest! {
        #[test]
        fn white_space_only_is_rejected(ws in r#"\s+"#) {
            let _ = ws.parse::<NameOfUser>().unwrap_err();
        }

        // It would probably be better to eliminate other sources
        // of confusion too, but control characters are the easiest since
        // supported by the rust language. I don't want to remove
        // punctation, since some names may include it.
        #[test]
        fn names_with_control_characters_are_rejected(cr in r#".*\p{Cc}+.*"#) {
            let _ = cr.parse::<NameOfUser>().unwrap_err();
        }
        #[test]
        fn mixing_scripts_are_accepted(name in r#"[\p{Greek}\p{Latin}\p{Arabic}\p{Hangul}]{1,60}"#) {
            let _ = name.parse::<NameOfUser>().unwrap();
        }
    }
}
