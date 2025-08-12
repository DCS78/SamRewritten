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

use crate::{dev_println, frontend::MainApplication};
use gtk::{AboutDialog, gio::SimpleAction, prelude::*};

/// Set up application actions and keyboard shortcuts.
pub fn setup_app_actions(
    application: &MainApplication,
    about_dialog: &AboutDialog,
    refresh_app_list_action: &SimpleAction,
    refresh_achievements_list_action: &SimpleAction,
    reset_all_stats_and_achievements_action: &SimpleAction,
) {
    let action_show_about_dialog = SimpleAction::new("about", None);
    action_show_about_dialog.connect_activate({
        let about_dialog = about_dialog.clone();
        move |_, _| {
            about_dialog.present();
        }
    });

    let action_quit = SimpleAction::new("quit", None);
    action_quit.connect_activate({
        let application = application.clone();
        move |_, _| {
            application.quit();
        }
    });

    for action in [
        refresh_app_list_action,
        refresh_achievements_list_action,
        reset_all_stats_and_achievements_action,
        &action_show_about_dialog,
        &action_quit,
    ] {
        application.add_action(action);
    }

    // Assign F5 to both refresh actions
    for accel in ["app.refresh_app_list", "app.refresh_achievements_list"] {
        application.set_accels_for_action(accel, &["F5"]);
    }
}

/// Enable or disable a named application action.
pub fn set_app_action_enabled(application: &MainApplication, action_name: &str, enabled: bool) {
    match application.lookup_action(action_name) {
        Some(action) => {
            if let Some(simple_action) = action.downcast_ref::<SimpleAction>() {
                simple_action.set_enabled(enabled);
            } else {
                dev_println!("[CLIENT] Action '{action_name}' is not a SimpleAction");
            }
        }
    _none => {
            dev_println!("[CLIENT] Action not found: {action_name}");
        }
    }
}
