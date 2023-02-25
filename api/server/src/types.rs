//! Utility Types
//!
//! This includes types that don't have a specific place, but are still usefull.
use std::fmt;

use serde::{Deserialize, Serialize};

/// Safely store passwords
///
/// Note that at the moment this does nothing - the goal is to provide
/// a type that can by default be logged without showing the password inside,
/// but allow for a development only flag/environmental variable to allow
/// logging.
///
/// This is safer then always remember to `skip` the private details in
/// for example [macro@tracing::instrument]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub(crate) struct Password(pub(crate) String);

/// Safely store emails
///
/// Does nothing at the moment but the goal is to partially anonymize
/// logged emails for privacy reasons while still allowing for development
/// to see them in clear, should also include some validation and canonicalization
/// (everything lowercase), but this is a task for future me.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct EMail(pub(crate) String);

impl fmt::Debug for EMail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
