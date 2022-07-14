use tracing::Level;
use tracing_subscriber::prelude::*;

pub fn init_tracing(service_name: &str, level: Level) -> anyhow::Result<()> {
    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name(service_name)
        .install_simple()?;

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let logging = tracing_subscriber::fmt::layer().pretty();

    let filter = tracing_subscriber::filter::Targets::new().with_target("seismic", level);

    tracing_subscriber::registry()
        .with(telemetry)
        .with(logging)
        .with(filter)
        .try_init()?;

    Ok(())
}
