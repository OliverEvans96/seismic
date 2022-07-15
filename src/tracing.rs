use tracing::Level;
use tracing_subscriber::prelude::*;

pub fn init_tracing(service_name: &str, level: Level, jaeger: bool) -> anyhow::Result<()> {
    let logging = tracing_subscriber::fmt::layer().pretty();

    let filter = tracing_subscriber::filter::Targets::new()
        .with_target("seismic", level)
        .with_target("server", level)
        .with_target("client", level);

    let subscriber_base = tracing_subscriber::registry().with(logging).with(filter);

    // Optionally send telemetry data to Jaeger
    if jaeger {
        let tracer = opentelemetry_jaeger::new_pipeline()
            .with_service_name(service_name)
            .install_simple()?;
        let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
        let subscriber = subscriber_base.with(telemetry);
        subscriber.try_init()?;
    } else {
        subscriber_base.try_init()?;
    }

    Ok(())
}
