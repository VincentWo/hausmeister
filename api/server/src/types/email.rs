use core::fmt;
use std::{ops::Deref, str::FromStr};

use color_eyre::{eyre::eyre, Report};
use serde::{Deserialize, Serialize};

/// Safely store & validate emails
///
/// Only allows valid emails in the sense of the webspec for the input[type=email]
/// field, this allows any commond email while forbidding things like comments etc.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "&str")]
pub(crate) struct EMail(String);
impl FromStr for EMail {
    type Err = Report;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: Parse
        if validator::validate_email(s) {
            Ok(Self(s.to_owned()))
        } else {
            Err(eyre!(
                "{s:?} is not a valid email address according to the HTML 5 Spec",
            ))
        }
    }
}

impl TryFrom<&str> for EMail {
    type Error = Report;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl AsRef<str> for EMail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for EMail {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl EMail {
    fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Debug for EMail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

#[cfg(test)]
mod tests {
    use super::EMail;
    use proptest::proptest;

    #[test]
    fn empty_string_is_rejected() {
        let _ = "".parse::<EMail>().unwrap_err();
    }

    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let _ = "your-domain.de".parse::<EMail>().unwrap_err();
    }

    #[test]
    fn email_missing_subject_is_rejected() {
        let _ = "@my-domain.com".parse::<EMail>().unwrap_err();
    }

    proptest! {
        #[test]
        fn email_parsing_does_not_crash(s in r#"\p{Cc}*"#) {
            let _ = s.parse::<EMail>();
        }

        // Regex is taken form the HTML 5 Spec +
        // length boundaries (64 for local part, 255 for domain) from RFC5321
        #[test]
        fn valid_emails_are_parsed_successfully(valid_email in r#"[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]{1,64}@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?){0,3}"#) {
            let _ = valid_email.parse::<EMail>().unwrap();
        }
    }
}
