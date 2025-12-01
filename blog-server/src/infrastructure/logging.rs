use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, fmt};

pub fn init_logging(format: String) {
    // setup logger globally.
    LogTracer::init().expect("failed to set LogTracer");

    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info,blog_server=debug"))
        .unwrap();

    if format == "json" {
        let subscriber = fmt()
            .with_env_filter(filter)
            .with_target(false)
            .with_level(true)
            .with_timer(fmt::time::UtcTime::rfc_3339())
            .json()
            .finish();

        let _ = tracing::subscriber::set_global_default(subscriber);
    } else {
        let subscriber = fmt()
            .with_env_filter(filter)
            .with_target(false)
            .with_level(true)
            .with_timer(fmt::time::UtcTime::rfc_3339())
            .finish();

        let _ = tracing::subscriber::set_global_default(subscriber);
    }
}
