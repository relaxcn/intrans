// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // 初始化 tracing subscriber
    // 在 debug 模式下使用 DEBUG 级别，release 模式使用 INFO 级别
    tracing_subscriber::fmt()
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| {
                    #[cfg(debug_assertions)]
                    {
                        "debug".into()
                    }
                    #[cfg(not(debug_assertions))]
                    {
                        "info".into()
                    }
                })
        )
        .init();

    tracing::info!("Intrans application starting");
    intrans_lib::run()
}
