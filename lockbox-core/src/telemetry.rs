use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::time::ChronoUtc;
use tracing_subscriber::{EnvFilter, Layer as TracingLayer};

pub use tracing::{error, info, warn, debug, trace, Span};


/// Setup the global logging configuration for the application.
/// It initializes the tracing subscriber with a JSON format,
/// using the environment variable `LOG_LEVEL` to set the log level.
pub fn setup_logging() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .flatten_event(true)
        .with_timer(ChronoUtc::rfc_3339())
        .with_span_events(FmtSpan::CLOSE)
        .with_ansi(false)
        .boxed();

    let env_filter = EnvFilter::try_from_env("LOG_LEVEL")
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}
