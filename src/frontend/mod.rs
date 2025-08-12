// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025 Paul <abonnementspaul (at) gmail.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

// --- Standard Library Imports ---
use std::sync::RwLock;

// --- External Crate Imports ---
use gtk::{glib::ExitCode, prelude::*};
use once_cell::sync::Lazy;

// --- Internal Crate Imports ---
use crate::APP_ID;
use crate::frontend::request::Request;
use crate::utils::bidir_child::BidirChild;
use app_list_view::create_main_ui;

/// Global state for the orchestrator process.
pub static DEFAULT_PROCESS: Lazy<RwLock<Option<BidirChild>>> = Lazy::new(|| RwLock::new(None));

// --- Module Declarations (alphabetical) ---
mod achievement;
mod achievement_automatic_view;
mod achievement_manual_view;
mod achievement_view;
mod app_list_view;
mod app_list_view_callbacks;
mod app_view;
mod application_actions;
mod custom_progress_bar_widget;
mod request;
mod shimmer_image;
mod stat;
mod stat_view;
mod steam_app;
mod ui_components;

// --- Main Application Logic ---
use request::Shutdown;

/// Gracefully shut down the orchestrator process.
fn shutdown() {
    if let Err(err) = Shutdown.request() {
        eprintln!("[CLIENT] Failed to send shutdown message: {}", err);
        return;
    }
    let mut guard = DEFAULT_PROCESS.write().unwrap();
    match &mut *guard {
        Some(bidir) => {
            bidir
                .child
                .wait()
                .expect("[CLIENT] Failed to wait on orchestrator to shutdown");
        }
        none => panic!("[CLIENT] No orchestrator process to shutdown"),
    }
}

#[cfg(not(feature = "adwaita"))]
pub type MainApplication = gtk::Application;
#[cfg(feature = "adwaita")]
pub type MainApplication = adw::Application;

/// Entry point for the main UI, sets up the application and event loop.
pub fn main_ui(orchestrator: BidirChild) -> ExitCode {
    *DEFAULT_PROCESS.write().unwrap() = Some(orchestrator);
    let main_app = MainApplication::builder()
        .application_id(APP_ID)
        .flags(
            gtk::gio::ApplicationFlags::HANDLES_COMMAND_LINE
                | gtk::gio::ApplicationFlags::NON_UNIQUE,
        )
        .build();

    // Set Adwaita color scheme to match OS theme
    #[cfg(feature = "adwaita")]
    {
        use adw::StyleManager;
        #[cfg(target_os = "windows")]
        {
            // Try to detect Windows dark mode via registry
            let is_dark = {
                use std::io::ErrorKind;
                let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
                let path = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize";
                match hkcu.open_subkey(path) {
                    Ok(key) => {
                        // 0 = dark, 1 = light
                        match key.get_value::<u32, _>("AppsUseLightTheme") {
                            Ok(val) => val == 0,
                            Err(_) => false,
                        }
                    }
                    Err(e) if e.kind() == ErrorKind::NotFound => false,
                    Err(_) => false,
                }
            };
            if is_dark {
                StyleManager::default().set_color_scheme(adw::ColorScheme::PreferDark);
            } else {
                StyleManager::default().set_color_scheme(adw::ColorScheme::PreferLight);
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            StyleManager::default().set_color_scheme(adw::ColorScheme::Default);
        }
    }

    main_app.connect_command_line(|app, cmd| create_main_ui(app, cmd));
    main_app.connect_shutdown(move |_| shutdown());
    main_app.run()
}
