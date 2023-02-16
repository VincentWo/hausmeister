//! Setup of webauthn
use webauthn_rs::{
    prelude::{Url, WebauthnError},
    Webauthn, WebauthnBuilder,
};

use crate::settings::AppConfig;

#[tracing::instrument]
pub(crate) fn setup(config: &AppConfig) -> Result<Webauthn, WebauthnError> {
    let rp_id = "localhost";

    // Allow since we are parsing a constant
    #[allow(clippy::unwrap_used)]
    let rp_origin = Url::parse("http://localhost:3779").unwrap();

    let mut builder = WebauthnBuilder::new(rp_id, &rp_origin)?.rp_name("Hausmeister");

    if config.allow_localhost {
        builder = builder.allow_any_port(true);
    }

    for origin in &config.allowed_origins {
        builder = builder.append_allowed_origin(origin);
    }

    builder.build()
}
