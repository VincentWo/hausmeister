use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub(crate) struct Password(pub(crate) String);

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct EMail(pub(crate) String);

impl fmt::Debug for EMail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
