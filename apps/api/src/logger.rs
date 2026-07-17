use tracing_subscriber::{EnvFilter, fmt, prelude::*};

pub fn init() {
    // RUST_LOG=info,my_app=debug,tower_http=debug
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,tower_http=debug,axum::rejection=trace"));

    let env = std::env::var("APP_ENV").unwrap_or_else(|_| "dev".into());

    let registry = tracing_subscriber::registry().with(filter);

    match env.as_str() {
        "production" | "prod" => {
            // JSON có structure, dễ ship qua Loki/Datadog/CloudWatch
            registry
                .with(
                    fmt::layer()
                        .json()
                        .with_current_span(true)
                        .with_span_list(false)
                        .with_target(true)
                        .with_file(false)
                        .with_line_number(false),
                )
                .init();
        }
        _ => {
            // Dev: đẹp, có màu, có file:line
            registry
                .with(
                    fmt::layer()
                        .pretty()
                        .with_target(true)
                        .with_file(true)
                        .with_line_number(true)
                        .with_thread_ids(false),
                )
                .init();
        }
    }
}
