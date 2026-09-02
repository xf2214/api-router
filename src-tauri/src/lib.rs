use std::time::Duration;

use tauri::Manager;
use tracing::info;

#[cfg(target_os = "macos")]
use tauri::menu::{Menu, PredefinedMenuItem, Submenu};

pub mod core;
pub mod infra;
pub mod server;
pub mod tauri_impl;

mod config;
mod error;
mod keyring;
mod state;
mod trace;

use state::AppState;

// ── Compatibility re-exports: old crate-level module paths keep working ──

pub mod router {
    pub use crate::core::*;
}

pub mod cache {
    pub use crate::infra::cache::*;
}

pub mod circuit {
    pub use crate::infra::circuit::*;
}

pub mod client {
    pub use crate::infra::client::*;
}

pub mod transform {
    pub use crate::infra::transform::*;
}

pub mod metrics {
    pub use crate::infra::metrics::*;
}

pub mod sse {
    pub use crate::server::sse::*;
}

pub mod commands {
    pub use crate::tauri_impl::*;
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_tracing();

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            tauri_impl::commands_config::get_config,
            tauri_impl::commands_config::export_config,
            tauri_impl::commands_config::save_config,
            tauri_impl::commands_provider::delete_provider,
            tauri_impl::commands_models::get_groups,
            tauri_impl::commands_models::save_group,
            tauri_impl::commands_models::delete_group,
            tauri_impl::commands_models::get_model_definitions,
            tauri_impl::commands_models::save_model_definition,
            tauri_impl::commands_models::delete_model_definition,
            tauri_impl::commands_server::start_server,
            tauri_impl::commands_server::stop_server,
            tauri_impl::commands_server::get_server_status,
            tauri_impl::commands_server::test_model_connection,
            tauri_impl::commands_server::test_model_config,
            tauri_impl::commands_provider::check_provider_health,
            tauri_impl::commands_provider::check_all_providers_health,
            tauri_impl::commands_provider::get_health_status,
            tauri_impl::commands_provider::fetch_provider_models,
            tauri_impl::commands_observability::get_request_logs,
            tauri_impl::commands_observability::get_request_stats,
            tauri_impl::commands_observability::clear_request_logs,
            tauri_impl::commands_provider::test_provider_target,
        ])
        .setup(|app| {
            let app_data_dir = app.path().app_local_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;
            let config_path = app_data_dir.join("config.yaml");
            let config = config::load(&config_path).unwrap_or_default();

            info!(
                "Loaded configuration from {} ({} providers, {} models)",
                config_path.display(),
                config.providers.len(),
                config.models.len()
            );

            let state = AppState::new(config, config_path);
            app.manage(state);

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state = handle.state::<AppState>();
                if let Err(e) = tauri_impl::commands_server::start_server(state).await {
                    tracing::error!("Auto-start server failed: {}", e);
                }
            });

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state = handle.state::<AppState>();
                loop {
                    let (enabled, interval) = {
                        let config = state.inner.config.read().await;
                        (
                            config.enable_auto_health_check,
                            config.health_check_interval_seconds,
                        )
                    };
                    if enabled {
                        tauri_impl::commands_provider::check_all_providers_health_internal(&state)
                            .await;
                    }
                    tokio::time::sleep(Duration::from_secs(interval.max(60))).await;
                }
            });
            Ok(())
        });

    #[cfg(target_os = "macos")]
    let builder = builder.menu(|app| build_menu(app));

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 初始化全局日志订阅者。
///
/// 优先尊重 `RUST_LOG` 环境变量；未设置时回退到 `info` 级别。
/// `try_init` 保证幂等，重复调用不会 panic。
fn init_tracing() {
    use tracing_subscriber::EnvFilter;

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init();
}

#[cfg(target_os = "macos")]
fn build_menu<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<Menu<R>> {
    let app_menu = Submenu::with_items(
        app,
        "API Router",
        true,
        &[
            &PredefinedMenuItem::about(app, Some("关于 API Router"), None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::hide(app, Some("隐藏 API Router"))?,
            &PredefinedMenuItem::hide_others(app, Some("隐藏其他"))?,
            &PredefinedMenuItem::show_all(app, Some("显示全部"))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::quit(app, Some("退出 API Router"))?,
        ],
    )?;

    let edit_menu = Submenu::with_items(
        app,
        "编辑",
        true,
        &[
            &PredefinedMenuItem::cut(app, None)?,
            &PredefinedMenuItem::copy(app, None)?,
            &PredefinedMenuItem::paste(app, None)?,
            &PredefinedMenuItem::select_all(app, None)?,
        ],
    )?;

    let window_menu = Submenu::with_items(
        app,
        "窗口",
        true,
        &[
            &PredefinedMenuItem::minimize(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::close_window(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::bring_all_to_front(app, Some("前置全部窗口"))?,
            &PredefinedMenuItem::fullscreen(app, Some("进入全屏"))?,
        ],
    )?;

    let help_menu = Submenu::with_items(app, "帮助", true, &[])?;

    Menu::with_items(app, &[&app_menu, &edit_menu, &window_menu, &help_menu])
}
