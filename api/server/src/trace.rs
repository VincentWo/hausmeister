//! Tracing registration

use color_eyre::Report;
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

/// Sets alle tracing subscriber up
///
/// At the moment this just registers the infrastructure for
/// span traces, env filters and pretty printing
///
/// The plan is to include opentelemetry support later on and
/// more config options
pub(crate) fn setup() -> Result<(), Report> {
    Registry::default()
        .with(EnvFilter::from_default_env())
        .with(
            HierarchicalLayer::new(2)
                .with_targets(true)
                .with_bracketed_fields(true),
        )
        .with(ErrorLayer::default())
        .init();

    Ok(())
}
