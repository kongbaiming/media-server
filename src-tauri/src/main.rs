// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use media_server::spawn_server;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{
    AppHandle, CustomMenuItem, LogicalPosition, LogicalSize, Manager, Menu, MenuItem, Submenu,
    SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem, WindowEvent,
};

#[derive(Serialize, Deserialize, Clone)]
struct WindowState {
    width: f64,
    height: f64,
    x: f64,
    y: f64,
    maximized: bool,
}

fn main() {
    // Run the Axum backend in a background thread so the main thread is free
    // for the Tauri/WebView event loop. The server keeps living even when the
    // window is hidden to the system tray.
    spawn_server();

    tauri::Builder::default()
        .menu(build_app_menu())
        // The tray icon and macOS-specific tray options come from
        // `tauri.conf.json -> tauri.systemTray.iconPath`; we only attach the
        // menu and event handler programmatically.
        .system_tray(
            SystemTray::new()
                .with_menu(build_tray_menu())
                .with_tooltip("MediaVault — local media server"),
        )
        .on_menu_event(handle_menu_event)
        .on_system_tray_event(handle_tray_event)
        .setup(|app| {
            restore_window_state(app);
            Ok(())
        })
        .on_window_event(handle_window_event)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn build_app_menu() -> Menu {
    let file = Submenu::new(
        "File",
        Menu::new()
            .add_item(
                CustomMenuItem::new("open_settings", "Settings...")
                    .accelerator("CmdOrCtrl+,"),
            )
            .add_native_item(MenuItem::Separator)
            .add_item(
                CustomMenuItem::new("quit", "Quit MediaVault").accelerator("CmdOrCtrl+Q"),
            ),
    );

    let edit = Submenu::new(
        "Edit",
        Menu::new()
            .add_native_item(MenuItem::Undo)
            .add_native_item(MenuItem::Redo)
            .add_native_item(MenuItem::Separator)
            .add_native_item(MenuItem::Cut)
            .add_native_item(MenuItem::Copy)
            .add_native_item(MenuItem::Paste)
            .add_native_item(MenuItem::SelectAll),
    );

    let view = Submenu::new(
        "View",
        Menu::new()
            .add_native_item(MenuItem::EnterFullScreen)
            .add_native_item(MenuItem::Separator)
            .add_item(CustomMenuItem::new("reload", "Reload").accelerator("CmdOrCtrl+R"))
            .add_item(
                CustomMenuItem::new("toggle_devtools", "Toggle Developer Tools")
                    .accelerator("CmdOrCtrl+Shift+I"),
            ),
    );

    let help = Submenu::new(
        "Help",
        Menu::new().add_item(CustomMenuItem::new("about", "About MediaVault")),
    );

    Menu::new()
        .add_submenu(file)
        .add_submenu(edit)
        .add_submenu(view)
        .add_submenu(help)
}

fn build_tray_menu() -> SystemTrayMenu {
    SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("show", "Show MediaVault"))
        .add_item(CustomMenuItem::new("hide", "Hide Window"))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("quit", "Quit"))
}

fn handle_menu_event(event: tauri::WindowMenuEvent) {
    let window = event.window().clone();
    let app = window.app_handle();
    match event.menu_item_id() {
        "open_settings" => navigate_to(&window, "/settings"),
        "quit" => app.exit(0),
        "reload" => {
            let _ = window.eval("window.location.reload()");
        }
        "toggle_devtools" => {
            #[cfg(debug_assertions)]
            {
                if window.is_devtools_open() {
                    window.close_devtools();
                } else {
                    window.open_devtools();
                }
            }
            #[cfg(not(debug_assertions))]
            {
                let _ = window.eval(
                    "console.info('DevTools are only available in debug builds')",
                );
            }
        }
        "about" => {
            let _ = window.eval(
                "alert('MediaVault v0.1.0\\n\\nLocal media server with Douyin support.\\nBackend runs on http://127.0.0.1:8080.')",
            );
        }
        _ => {}
    }
}

fn handle_tray_event(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::DoubleClick { .. } => toggle_window_visibility(app),
        SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
            "show" => show_window(app),
            "hide" => hide_window(app),
            "quit" => app.exit(0),
            _ => {}
        },
        _ => {}
    }
}

fn handle_window_event(event: tauri::GlobalWindowEvent) {
    match event.event() {
        // The window's close button hides the window to the system tray
        // instead of quitting. The server keeps running in the background.
        // Quit explicitly via the menu or tray "Quit" entry.
        WindowEvent::CloseRequested { api, .. } => {
            save_window_state(event.window());
            api.prevent_close();
            let _ = event.window().hide();
        }
        WindowEvent::Resized(_) | WindowEvent::Moved(_) => {
            save_window_state(event.window());
        }
        _ => {}
    }
}

fn show_window(app: &AppHandle) {
    if let Some(window) = app.get_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn hide_window(app: &AppHandle) {
    if let Some(window) = app.get_window("main") {
        let _ = window.hide();
    }
}

fn toggle_window_visibility(app: &AppHandle) {
    if let Some(window) = app.get_window("main") {
        match window.is_visible() {
            Ok(true) => {
                let _ = window.hide();
            }
            _ => show_window(app),
        }
    }
}

fn navigate_to<R: tauri::Runtime>(window: &tauri::Window<R>, path: &str) {
    let _ = window.show();
    let _ = window.unminimize();
    let _ = window.set_focus();
    // HashRouter listens to hashchange, so setting the hash is enough.
    let js = format!("window.location.hash = '#{}'", path);
    let _ = window.eval(&js);
}

// -- Window state persistence -------------------------------------------------

fn state_path() -> Option<PathBuf> {
    let mut path = dirs::data_dir()?;
    path.push(if cfg!(target_os = "macos") {
        "com.mediavault.app"
    } else {
        "MediaVault"
    });
    path.push("window.json");
    Some(path)
}

fn load_window_state() -> Option<WindowState> {
    let path = state_path()?;
    let text = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&text).ok()
}

fn restore_window_state<R: tauri::Runtime>(app: &tauri::App<R>) {
    let Some(state) = load_window_state() else { return };
    let Some(window) = app.get_window("main") else { return };

    // Skip restoring to a position that lies entirely outside any current
    // monitor — this can happen when a previously-attached display is gone.
    if !position_is_on_screen(state.x, state.y, state.width, state.height, &window) {
        return;
    }

    let _ = window.set_size(LogicalSize::new(state.width, state.height));
    let _ = window.set_position(LogicalPosition::new(state.x, state.y));
    if state.maximized {
        let _ = window.maximize();
    }
}

fn save_window_state<R: tauri::Runtime>(window: &tauri::Window<R>) {
    // Don't overwrite a saved state with one captured while the window is
    // hidden in the tray — at that point the OS may report 0x0.
    if !window.is_visible().unwrap_or(false) {
        return;
    }

    let maximized = window.is_maximized().unwrap_or(false);
    let size = window.outer_size().ok();
    let pos = window.outer_position().ok();

    if let (Some(s), Some(p)) = (size, pos) {
        let s = s.to_logical::<f64>(1.0);
        let p = p.to_logical::<f64>(1.0);
        // Don't persist a zero-size window (happens mid-minimize).
        if s.width <= 0.0 || s.height <= 0.0 {
            return;
        }
        let state = WindowState {
            width: s.width,
            height: s.height,
            x: p.x,
            y: p.y,
            maximized,
        };
        if let Ok(json) = serde_json::to_string(&state) {
            if let Some(path) = state_path() {
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::write(path, json);
            }
        }
    }
}

fn position_is_on_screen<R: tauri::Runtime>(
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    window: &tauri::Window<R>,
) -> bool {
    let Ok(monitors) = window.available_monitors() else {
        return true;
    };
    let win_left = x;
    let win_top = y;
    let win_right = x + w;
    let win_bottom = y + h;
    monitors.iter().any(|m| {
        let scale = m.scale_factor();
        let pos = m.position().to_logical::<f64>(scale);
        let size = m.size().to_logical::<f64>(scale);
        let mon_left = pos.x;
        let mon_top = pos.y;
        let mon_right = pos.x + size.width;
        let mon_bottom = pos.y + size.height;
        // Require a meaningful overlap rather than just any touch.
        let overlap_w = win_right.min(mon_right) - win_left.max(mon_left);
        let overlap_h = win_bottom.min(mon_bottom) - win_top.max(mon_top);
        overlap_w > 50.0 && overlap_h > 50.0
    })
}
