
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
use once_cell::sync::Lazy;
use gtk::{glib::ExitCode, prelude::*};

// --- Internal Crate Imports ---
use crate::APP_ID;
use crate::frontend::request::Request;
use crate::utils::bidir_child::BidirChild;
use app_list_view::create_main_ui;

// --- Global State ---
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

fn shutdown() {
    if let Err(err) = Shutdown.request() {
        eprintln!("[CLIENT] Failed to send shutdown message: {}", err);
        return;
    }
    let mut guard = DEFAULT_PROCESS.write().unwrap();
    match &mut *guard {
        Some(bidir) => {
            bidir.child.wait().expect("[CLIENT] Failed to wait on orchestrator to shutdown");
        }
        None => panic!("[CLIENT] No orchestrator process to shutdown"),
    }
}

#[cfg(not(feature = "adwaita"))]
pub type MainApplication = gtk::Application;
#[cfg(feature = "adwaita")]
pub type MainApplication = adw::Application;

pub fn main_ui(orchestrator: BidirChild) -> ExitCode {
    *DEFAULT_PROCESS.write().unwrap() = Some(orchestrator);
    let main_app = MainApplication::builder()
        .application_id(APP_ID)
        .flags(gtk::gio::ApplicationFlags::HANDLES_COMMAND_LINE | gtk::gio::ApplicationFlags::NON_UNIQUE)
        .build();
    main_app.connect_command_line(|app, cmd| create_main_ui(app, cmd));
    main_app.connect_shutdown(move |_| shutdown());
    main_app.run()
}
