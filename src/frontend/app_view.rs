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

use super::stat_view::create_stats_view;
use crate::frontend::MainApplication;
use crate::frontend::achievement_view::create_achievements_view;
use crate::frontend::shimmer_image::ShimmerImage;
use gtk::gio::ListStore;
use gtk::glib::clone;
use gtk::pango::{EllipsizeMode, WrapMode};
use gtk::prelude::*;
use gtk::{
    Adjustment, Align, Box, Button, Label, Orientation, Separator, SpinButton, Spinner, Stack,
    StackTransitionType, StringFilter, ToggleButton,
};
use gtk::{Paned, glib};
use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

enum AppStackPage {
    Achievements,
    Stats,
    Failed,
    Empty,
    Loading,
}

impl AppStackPage {
    fn as_str(&self) -> &'static str {
        match self {
            AppStackPage::Achievements => "achievements",
            AppStackPage::Stats => "stats",
            AppStackPage::Failed => "failed",
            AppStackPage::Empty => "empty",
            AppStackPage::Loading => "loading",
        }
    }
}

/// Create the main app view, including sidebar, achievements, and stats.
pub fn create_app_view(
    app_id: Rc<Cell<Option<u32>>>,
    app_unlocked_achievements_count: Rc<Cell<usize>>,
    application: &MainApplication,
) -> (
    Stack,
    ShimmerImage,
    Label,
    ToggleButton,
    ToggleButton,
    Label,
    Label,
    Label,
    Label,
    Label,
    Box,
    Box,
    ListStore,
    StringFilter,
    ListStore,
    StringFilter,
    Paned,
    Adjustment,
    SpinButton,
    Button,
    Arc<AtomicBool>,
    Stack,
) {
    let app_spinner = Spinner::builder().spinning(true).margin_end(5).build();
    let app_spinner_label = Label::builder().label("Loading...").build();
    let app_spinner_box = Box::builder().halign(Align::Center).build();
    app_spinner_box.append(&app_spinner);
    app_spinner_box.append(&app_spinner_label);

    let _app_achievement_count_label = Label::builder()
        .label("Achievements:")
        .halign(Align::Start)
        .build();
    let app_achievement_count_value = Label::builder().halign(Align::End).build();
    let app_achievement_count_box = create_labeled_value_box(
        "Achievements:",
        &app_achievement_count_value,
        10,
    );

    let app_stats_count_value = Label::builder().halign(Align::End).build();
    let app_stats_count_box = create_labeled_value_box("Stats:", &app_stats_count_value, 10);

    let app_type_value = Label::builder().halign(Align::End).build();
    let app_type_box = create_labeled_value_box("Type:", &app_type_value, 10);

    let app_developer_value = Label::builder()
        .halign(Align::End)
        .ellipsize(EllipsizeMode::End)
        .build();
    let app_developer_box = create_labeled_value_box("Developer:", &app_developer_value, 20);

    let app_metacritic_value = Label::builder().halign(Align::End).build();
    let app_metacritic_box = create_labeled_value_box("Metacritic:", &app_metacritic_value, 10);

    let app_loading_failed_label = Label::builder()
        .label("Failed to load app.")
        .halign(Align::Center)
        .valign(Align::Center)
        .build();

    let app_no_entries_value = Label::builder()
        .label("No entries found.")
        .halign(Align::Center)
        .valign(Align::Center)
        .build();

    let app_label = Label::builder()
        .margin_top(20)
        .wrap(true)
        .wrap_mode(WrapMode::WordChar)
        .halign(Align::Start)
        .build();

    let app_shimmer_image = ShimmerImage::new();
    app_shimmer_image.set_halign(Align::Start);

    let app_achievements_button = ToggleButton::builder().label("Achievements").build();
    let app_stats_button = ToggleButton::builder()
        .label("Stats")
        .group(&app_achievements_button)
        .build();
    let app_button_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["linked"].as_slice())
        .margin_bottom(20)
        .margin_start(0)
        .homogeneous(true)
        .margin_end(0)
        .width_request(231)
        .halign(Align::Start)
        .build();
    app_button_box.append(&app_achievements_button);
    app_button_box.append(&app_stats_button);

    let app_sidebar_separator = Separator::builder()
        .orientation(Orientation::Horizontal)
        .margin_top(20)
        .build();

    let app_sidebar = Box::builder()
        .orientation(Orientation::Vertical)
        .margin_top(20)
        .margin_bottom(20)
        .margin_start(20)
        .margin_end(20)
        .build();
    app_sidebar.append(&app_button_box);
    app_sidebar.append(&app_shimmer_image);
    app_sidebar.append(&app_label);
    app_sidebar.append(&app_sidebar_separator);
    app_sidebar.append(&app_developer_box);
    app_sidebar.append(&app_metacritic_box);
    app_sidebar.append(&app_achievement_count_box);
    app_sidebar.append(&app_stats_count_box);
    app_sidebar.append(&app_type_box);

    let (
        app_achievements_stack,
        app_achievements_model,
        app_achievement_string_filter,
        achievements_manual_adjustement,
        achievements_manual_spinbox,
        achievements_manual_start,
        cancel_timed_unlock,
    ) = create_achievements_view(
        app_id.clone(),
        app_unlocked_achievements_count,
        application,
        &app_achievement_count_value,
    );

    let (app_stat_scrolled_window, app_stat_model, app_stat_string_filter) = create_stats_view();

    let app_stack = Stack::builder()
        .transition_type(StackTransitionType::SlideLeftRight)
        .build();
    app_stack.add_named(&app_achievements_stack, Some(AppStackPage::Achievements.as_str()));
    app_stack.add_named(&app_stat_scrolled_window, Some(AppStackPage::Stats.as_str()));
    app_stack.add_named(&app_loading_failed_label, Some(AppStackPage::Failed.as_str()));
    app_stack.add_named(&app_no_entries_value, Some(AppStackPage::Empty.as_str()));
    app_stack.add_named(&app_spinner_box, Some(AppStackPage::Loading.as_str()));

    app_stack.connect_visible_child_name_notify(clone!(
        #[weak]
        app_achievements_button,
        #[weak]
        app_stats_button,
        move |stack| {
            match stack.visible_child_name().as_deref() {
                Some(x) if x == AppStackPage::Loading.as_str() || x == AppStackPage::Failed.as_str() => {
                    app_achievements_button.set_sensitive(false);
                    app_stats_button.set_sensitive(false);
                }
                Some(x) if x == AppStackPage::Achievements.as_str() => {
                    app_achievements_button.set_active(true);
                    app_stats_button.set_active(false);
                    app_achievements_button.set_sensitive(true);
                    app_stats_button.set_sensitive(true);
                }
                _ => {
                    app_achievements_button.set_active(false);
                    app_stats_button.set_active(true);
                    app_achievements_button.set_sensitive(true);
                    app_stats_button.set_sensitive(true);
                }
            }
        }
    ));

    app_achievements_button.connect_clicked(clone!(
        #[weak]
        app_stack,
        #[weak]
        app_achievements_model,
        move |_| {
            if app_achievements_model.n_items() == 0 {
                app_stack.set_visible_child_name(AppStackPage::Empty.as_str());
            } else {
                app_stack.set_visible_child_name(AppStackPage::Achievements.as_str());
            }
        }
    ));

    app_stats_button.connect_clicked(clone!(
        #[weak]
        app_stack,
        #[weak]
        app_stat_model,
        move |_| {
            if app_stat_model.n_items() == 0 {
                app_stack.set_visible_child_name(AppStackPage::Empty.as_str());
            } else {
                app_stack.set_visible_child_name(AppStackPage::Stats.as_str());
            }
        }
    ));

    // Create app pane with sidebar and main content
    let app_pane = Paned::builder()
        .orientation(Orientation::Horizontal)
        .shrink_start_child(false)
        .shrink_end_child(false)
        .resize_start_child(false)
        .start_child(&app_sidebar)
        .end_child(&app_stack)
        .build();

    // Return relevant widgets that need to be accessed from outside
    (
        app_stack,
        app_shimmer_image,
        app_label,
        app_achievements_button,
        app_stats_button,
        app_achievement_count_value,
        app_stats_count_value,
        app_type_value,
        app_developer_value,
        app_metacritic_value,
        app_metacritic_box,
        app_sidebar,
        app_achievements_model,
        app_achievement_string_filter,
        app_stat_model,
        app_stat_string_filter,
        app_pane,
        achievements_manual_adjustement,
        achievements_manual_spinbox,
        achievements_manual_start,
        cancel_timed_unlock,
        app_achievements_stack,
    )
}

fn create_labeled_value_box(label: &str, value: &gtk::Label, margin_top: i32) -> gtk::Box {
    let label_widget = gtk::Label::builder().label(label).halign(gtk::Align::Start).build();
    let spacer = gtk::Box::builder().hexpand(true).build();
    let box_widget = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .margin_top(margin_top)
        .build();
    box_widget.append(&label_widget);
    box_widget.append(&spacer);
    box_widget.append(value);
       box_widget
   }
