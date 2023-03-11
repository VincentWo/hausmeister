use std::str::FromStr;

use argon2::{
    password_hash::{self, rand_core::OsRng, PasswordHashString, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use color_eyre::{eyre::eyre, Report};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;

/// Safely store passwords
///
/// Prevents logging or plain-text comparision, ensures that the minimum
/// length of 10 bytes is followed
///
///
/// This is safer then always remember to `skip` the private details in
/// for example [macro@tracing::instrument]
#[derive(Debug, Deserialize)]
#[serde(try_from = "&str")]
pub(crate) struct Password(SecretString);

impl FromStr for Password {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.len() < 10 {
            Err(eyre!("{s:?} is shorter than 10 characters/bytes"))
        } else if s.len() > 256 {
            Err(eyre!("{s:?} is longer than 256 characters/bytes"))
        } else {
            Ok(Self(s.to_owned().into()))
        }
    }
}
impl TryFrom<&str> for Password {
    type Error = <Password as FromStr>::Err;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl Password {
    pub(crate) fn expose(&self) -> &str {
        self.0.expose_secret()
    }
    pub(crate) fn hash(&self) -> Result<PasswordHashString, Report> {
        let argon2 = Argon2::default();
        // TODO: Is this safe?
        let salt = SaltString::generate(&mut OsRng);
        let hash = argon2.hash_password(self.expose().as_bytes(), &salt)?;
        let hash = hash.to_owned();

        Ok(hash.serialize())
    }
    pub(crate) fn match_hash(&self, hash: &str) -> Result<bool, Report> {
        let hash = PasswordHash::new(hash)?;

        Argon2::default()
            .verify_password(self.expose().as_bytes(), &hash)
            .map(|_| true)
            .or_else(|e| match e {
                password_hash::Error::Password => Ok(false),
                e => Err(e.into()),
            })
    }
}

#[cfg(test)]
mod tests {
    use proptest::proptest;

    use super::Password;

    #[test]
    fn empty_password_is_rejected() {
        let _ = "".parse::<Password>().unwrap_err();
    }

    proptest! {
        #[test]
        fn whitespace_only_is_rejected(ws in r#"\s*"#) {
            let _ = ws.parse::<Password>().unwrap_err();
        }

        // Note that we only test short ascii password since the server
        // checks for byte length. This is actually intendened since in
        // some languages short passwords already give good entropy.
        #[test]
        fn short_passwords_are_rejected(short in r#"\p{ascii}{0, 9}"#) {
            let _ = short.parse::<Password>().unwrap_err();
        }

        // Come as you are, 256 bytes maximum, so 256/4 = 64 maximum
        #[test]
        fn everything_goes(s in r#".{64}"#) {
            let _ = s.parse::<Password>().unwrap();
        }

        #[test]
        fn allow_64_4_byte_scalars(s in r#"\p{Cuneiform}{64}"#) {
            let _ = s.parse::<Password>().unwrap();
        }
    }
}
