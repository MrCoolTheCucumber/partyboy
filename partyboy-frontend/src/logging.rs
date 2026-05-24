use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logger(enable_file_logging: bool) {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("info")
            .add_directive("wgpu_core=warn".parse().unwrap())
            .add_directive("wgpu_hal=error".parse().unwrap())
            .add_directive("naga=warn".parse().unwrap())
    });

    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false);

    let registry = tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer);

    if enable_file_logging {
        // Non-blocking file writer so we don't stall the emulator thread on disk I/O.
        let file_appender = tracing_appender::rolling::never("log", "output.log");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        // Keep the guard alive for the lifetime of the program.
        // Leaking is acceptable and common for long-running binaries.
        std::mem::forget(guard);

        let file_layer = fmt::layer()
            .with_writer(non_blocking)
            .with_ansi(false)
            .with_target(true);

        registry.with(file_layer).init();
    } else {
        registry.init();
    }

    install_panic_hook();
}

fn install_panic_hook() {
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        tracing::error!(panic = ?panic_info, "application panicked");

        // Delegate to the previous hook so the usual panic output still happens
        prev_hook(panic_info);
    }));
}
