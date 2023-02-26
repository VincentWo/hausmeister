//! All final request handlers
//!
//! These handlers return the actual responses, semantically grouped
pub(crate) mod healthcheck;
pub(crate) mod login;
pub(crate) mod register;
pub(crate) mod reset;
pub(crate) mod user;
pub(crate) mod webauthn;
