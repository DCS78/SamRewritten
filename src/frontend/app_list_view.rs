use gtk::glib::translate::FromGlib;
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

use crate::{
    backend::app_lister::{AppModel, AppModelType},
    frontend::{
        MainApplication,
        achievement::GAchievementObject,
        app_list_view_callbacks::switch_from_app_list_to_app,
        app_view::create_app_view,
        application_actions::{set_app_action_enabled, setup_app_actions},
        request::{GetAchievements, GetOwnedAppList, GetStats, Request, ResetStats, StopApp},
        shimmer_image::ShimmerImage,
        stat::GStatObject,
        steam_app::GSteamAppObject,
        ui_components::{
            create_about_dialog, create_context_menu_button,
            set_context_popover_to_app_list_context,
        },
    },
    utils::{arguments::parse_gui_arguments, ipc_types::SamError},
};
use gtk::glib::SignalHandlerId;
use gtk::{
    Align, ApplicationWindow, Box, Button, FilterListModel, HeaderBar, IconSize, Image, Label,
    ListItem, ListView, NoSelection, Orientation, PolicyType, ScrolledWindow, SearchEntry,
    SignalListItemFactory, Spinner, Stack, StackTransitionType, StringFilter,
    StringFilterMatchMode, Widget,
    gio::{ApplicationCommandLine, ListStore, SimpleAction, spawn_blocking},
    glib::{self, ExitCode, MainContext, clone},
    prelude::*,
};
use std::os::raw::c_ulong;
use std::process::Command;
use std::{cell::Cell, rc::Rc};
pub fn create_main_ui(
    application: &MainApplication,
    cmd_line: &ApplicationCommandLine,
) -> ExitCode {
    let gui_args = parse_gui_arguments(cmd_line);
    let launch_app_by_id_visible = Rc::new(Cell::new(false));
    let app_id = Rc::new(Cell::new(Option::<u32>::None));
    let app_unlocked_achievements_count = Rc::new(Cell::new(0usize));

    // Create the UI components for the app view
    let (
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
        achievements_manual_adjustment,
        achievements_manual_spinbox,
        achievements_manual_start,
        cancel_timed_unlock,
        app_achievements_stack,
    ) = create_app_view(
        app_id.clone(),
        app_unlocked_achievements_count.clone(),
        application,
    );

    // Loading box
    let list_spinner = Spinner::builder().margin_end(5).spinning(true).build();
    let list_spinner_label = Label::builder().label("Loading...").build();
    let list_spinner_box = Box::builder().halign(Align::Center).build();
    list_spinner_box.append(&list_spinner);
    list_spinner_box.append(&list_spinner_label);

    // Empty search result box
    let app_list_no_result_icon = {
        let icon = Image::from_icon_name("edit-find-symbolic");
        icon.set_icon_size(IconSize::Large);
        icon
    };
    let app_list_no_result_label = Label::builder().build();
    let app_list_no_result_box = Box::builder()
        .spacing(20)
        .valign(Align::Center)
        .halign(Align::Center)
        .orientation(Orientation::Vertical)
        .build();
    app_list_no_result_box.append(&app_list_no_result_icon);
    app_list_no_result_box.append(&app_list_no_result_label);

    // Header bar
    let header_bar = HeaderBar::builder().show_title_buttons(true).build();
    let search_entry = SearchEntry::builder()
        .placeholder_text("App name or App Id")
        .build();
    let back_button = Button::builder()
        .icon_name("go-previous")
        .sensitive(false)
        .build();
    let (context_menu_button, _, menu_model) = create_context_menu_button();
    header_bar.pack_start(&back_button);
    header_bar.pack_start(&search_entry);
    header_bar.pack_end(&context_menu_button);

    let list_scrolled_window = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .min_content_width(360)
        .build();

    let list_of_apps_or_no_result = Stack::builder()
        .transition_type(StackTransitionType::Crossfade)
        .build();
    list_of_apps_or_no_result.add_named(&list_scrolled_window, Some("list"));
    list_of_apps_or_no_result.add_named(&app_list_no_result_box, Some("empty"));

    // Main application stack component
    let list_stack = Stack::builder()
        .transition_type(StackTransitionType::SlideLeftRight)
        .build();
    list_stack.add_named(&list_spinner_box, Some("loading"));
    list_stack.add_named(&list_of_apps_or_no_result, Some("list"));
    list_stack.add_named(&app_pane, Some("app"));

    // App list models
    let list_factory = SignalListItemFactory::new();
    let list_store = ListStore::new::<GSteamAppObject>();
    let list_string_filter = StringFilter::builder()
        .expression(&GSteamAppObject::this_expression("app_name"))
        .match_mode(StringFilterMatchMode::Substring)
        .ignore_case(true)
        .build();
    let list_filter_model = FilterListModel::builder()
        .model(&list_store)
        .filter(&list_string_filter)
        .build();
    let list_selection_model = {
        let model = NoSelection::new(Option::<ListStore>::None);
        model.set_model(Some(&list_filter_model));
        model
    };
    let list_view = ListView::builder()
        // .single_click_activate(true)
        .orientation(Orientation::Vertical)
        .show_separators(true)
        .model(&list_selection_model)
        .factory(&list_factory)
        .build();

    let window = ApplicationWindow::builder()
        .application(application)
        .title("SamRewritten")
        .default_width(800)
        .default_height(600)
        .child(&list_stack)
        .titlebar(&header_bar)
        .build();

    let about_dialog = create_about_dialog(&window);

    // Connect list view activation
    list_view.connect_activate(clone!(
        #[strong]
        app_id,
        #[weak]
        application,
        #[weak]
        menu_model,
        #[weak]
        app_achievement_count_value,
        #[weak]
        app_stats_count_value,
        #[weak]
        app_type_value,
        #[weak]
        app_developer_value,
        #[weak]
        app_metacritic_value,
        #[weak]
        app_metacritic_box,
        #[weak]
        app_stack,
        #[weak]
        list_stack,
        #[weak]
        app_label,
        #[weak]
        app_shimmer_image,
        move |list_view, position| {
            let Some(model) = list_view.model() else {
                return;
            };
            let Some(item) = model.item(position).and_downcast::<GSteamAppObject>() else {
                return;
            };

            switch_from_app_list_to_app(
                &item,
                application.clone(),
                &app_type_value,
                &app_developer_value,
                &app_achievement_count_value,
                &app_stats_count_value,
                app_stack.clone(),
                &app_id,
                &app_metacritic_box,
                &app_metacritic_value,
                &app_shimmer_image,
                &app_label,
                &menu_model,
                &list_stack,
            );
        }
    ));

    list_factory.connect_setup(move |_, list_item| {
        if let Some(list_item) = list_item.downcast_ref::<gtk::ListItem>() {
            setup_list_item(list_item);
        }
    });

    /// Helper to setup a list item row for the app list view.
    fn setup_list_item(list_item: &ListItem) {
        let image = ShimmerImage::new();
        let label = Label::builder().margin_start(20).build();
        let spacer = Box::builder()
            .orientation(Orientation::Horizontal)
            .hexpand(true)
            .build();

        let make_button = |icon_name: &str, label_text: &str| {
            let icon = Image::builder().icon_name(icon_name).pixel_size(11).build();
            let label = Label::builder().label(label_text).build();
            let box_ = Box::builder()
                .spacing(8)
                .margin_start(10)
                .margin_end(10)
                .build();
            box_.append(&icon);
            box_.append(&label);
            Button::builder()
                .child(&box_)
                .margin_top(20)
                .margin_bottom(20)
                .margin_end(20)
                .margin_start(20)
                .build()
        };
        let launch_button = make_button("media-playback-start-symbolic", "Launch");

        let manage_box = {
            let icon = Image::builder()
                .icon_name("document-edit-symbolic")
                .pixel_size(11)
                .build();
            let label = Label::builder().label("Manage").build();
            let box_ = Box::builder()
                .spacing(8)
                .margin_start(10)
                .margin_end(10)
                .build();
            box_.append(&icon);
            box_.append(&label);
            box_
        };
        let manage_button = Button::builder()
            .child(&manage_box)
            .css_classes(vec!["suggested-action"])
            .build();
        let manage_new_button = Button::builder().icon_name("window-new-symbolic").build();
        if let Some(child) = manage_new_button.child() {
            if let Ok(img) = child.downcast::<Image>() {
                img.set_pixel_size(11);
            }
        }
        let manage_button_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .css_classes(vec!["linked"])
            .margin_top(20)
            .margin_bottom(20)
            .margin_end(20)
            .build();
        manage_button_box.append(&manage_button);
        manage_button_box.append(&manage_new_button);

        let entry = Box::builder()
            .orientation(Orientation::Horizontal)
            .margin_top(4)
            .margin_bottom(4)
            .margin_start(8)
            .margin_end(8)
            .build();
        entry.append(&image);
        entry.append(&label);
        entry.append(&spacer);
        entry.append(&launch_button);
        entry.append(&manage_button_box);

        let list_item = list_item
            .downcast_ref::<ListItem>()
            .expect("Needs to be a ListItem");
        list_item.set_child(Some(&entry));
        list_item
            .property_expression("item")
            .chain_property::<GSteamAppObject>("app_name")
            .bind(&label, "label", Widget::NONE);
        list_item
            .property_expression("item")
            .chain_property::<GSteamAppObject>("image_url")
            .bind(&image, "url", Widget::NONE);
    }

    list_factory.connect_bind(clone!(
        #[strong]
        app_id,
        #[weak]
        application,
        #[weak]
        menu_model,
        #[weak]
        app_achievement_count_value,
        #[weak]
        app_stats_count_value,
        #[weak]
        app_type_value,
        #[weak]
        app_developer_value,
        #[weak]
        app_metacritic_value,
        #[weak]
        app_metacritic_box,
        #[weak]
        app_stack,
        #[weak]
        list_stack,
        #[weak]
        app_label,
        #[weak]
        app_shimmer_image,
        move |_, list_item| {
            let list_item = list_item
                .downcast_ref::<ListItem>()
                .expect("Needs to be a ListItem");
            let steam_app_object = list_item
                .item()
                .and_then(|item| item.downcast::<GSteamAppObject>().ok())
                .expect("Item should be a GSteamAppObject");
            let app_id_to_bind = steam_app_object.app_id();

            let manage_box = list_item
                .child()
                .and_then(|child| child.downcast::<Box>().ok())
                .and_then(|app_box| app_box.last_child())
                .and_then(|button_box| button_box.downcast::<Box>().ok());

            let manage_button = manage_box
                .clone()
                .and_then(|button_box| button_box.first_child())
                .and_then(|manage_button| manage_button.downcast::<Button>().ok())
                .expect("Could not find Manage button widget");

            let manage_button_new_window = manage_box
                .clone()
                .and_then(|button_box| button_box.last_child())
                .and_then(|manage_button| manage_button.downcast::<Button>().ok())
                .expect("Could not find Manage new window button widget");

            let launch_button = manage_box
                .and_then(|child| child.prev_sibling())
                .and_then(|launch_button| launch_button.downcast::<Button>().ok())
                .expect("Could not find Launch button widget");

            let handler = manage_button.connect_clicked(clone!(
                #[strong]
                app_id,
                move |_| {
                    switch_from_app_list_to_app(
                        &steam_app_object,
                        application.clone(),
                        &app_type_value,
                        &app_developer_value,
                        &app_achievement_count_value,
                        &app_stats_count_value,
                        app_stack.clone(),
                        &app_id,
                        &app_metacritic_box,
                        &app_metacritic_value,
                        &app_shimmer_image,
                        &app_label,
                        &menu_model,
                        &list_stack,
                    );
                }
            ));

            unsafe {
                manage_button.set_data("handler", handler.as_raw());
            }

            let handler = manage_button_new_window.connect_clicked(move |_| {
                use crate::get_executable_path;
                Command::new(get_executable_path())
                    .arg(&format!("--auto-open={app_id_to_bind}"))
                    .spawn()
                    .expect("Could not start child process");
            });

            unsafe {
                manage_button_new_window.set_data("handler", handler.as_raw());
            }

            let handler = launch_button.connect_clicked(move |_| {
                #[cfg(unix)]
                {
                    Command::new("xdg-open")
                        .arg(&format!("steam://run/{app_id_to_bind}"))
                        .spawn()
                        .expect("Could not start child process")
                        .wait()
                        .expect("Failed to wait on child process");
                }

                #[cfg(windows)]
                {
                    Command::new("cmd")
                        .arg("/C")
                        .arg("start")
                        .arg(&format!("steam://run/{app_id_to_bind}"))
                        .spawn()
                        .expect("Could not start child process")
                        .wait()
                        .expect("Failed to wait on child process");
                }
            });

            unsafe {
                launch_button.set_data("handler", handler.as_raw());
            }
        }
    ));

    list_factory.connect_unbind(move |_, list_item| {
        let list_item = list_item
            .downcast_ref::<ListItem>()
            .expect("Needs to be a ListItem");

        let manage_box = list_item
            .child()
            .and_then(|child| child.downcast::<Box>().ok())
            .and_then(|app_box| app_box.last_child())
            .and_then(|button_box| button_box.downcast::<Box>().ok());

        let manage_button = manage_box
            .clone()
            .and_then(|button_box| button_box.first_child())
            .and_then(|manage_button| manage_button.downcast::<Button>().ok())
            .expect("Could not find Manage button widget");

        let manage_button_new_window = manage_box
            .clone()
            .and_then(|button_box| button_box.last_child())
            .and_then(|manage_button| manage_button.downcast::<Button>().ok())
            .expect("Could not find Manage new window button widget");

        let launch_button = manage_box
            .and_then(|child| child.prev_sibling())
            .and_then(|launch_button| launch_button.downcast::<Button>().ok())
            .expect("Could not find Launch button widget");

        unsafe {
            if let Some(handler) = manage_button.data("handler") {
                let ulong: c_ulong = *handler.as_ptr();
                let signal_handler = SignalHandlerId::from_glib(ulong);
                manage_button.disconnect(signal_handler);
            } else {
                eprintln!("[CLIENT] Manage button unbind failed");
            }

            if let Some(handler) = manage_button_new_window.data("handler") {
                let ulong: c_ulong = *handler.as_ptr();
                let signal_handler = SignalHandlerId::from_glib(ulong);
                manage_button_new_window.disconnect(signal_handler);
            } else {
                eprintln!("[CLIENT] Manage button new window unbind failed");
            }

            if let Some(handler) = launch_button.data("handler") {
                let ulong: c_ulong = *handler.as_ptr();
                let signal_handler = SignalHandlerId::from_glib(ulong);
                launch_button.disconnect(signal_handler);
            } else {
                eprintln!("[CLIENT] Launch button unbind failed");
            }
        }
    });

    // Search entry setup
    search_entry.connect_search_changed(clone!(
        #[weak]
        list_string_filter,
        #[weak]
        app_stat_string_filter,
        #[weak]
        app_achievement_string_filter,
        #[weak]
        list_store,
        move |entry| {
            let text = Some(entry.text()).filter(|s| !s.is_empty());

            // This logic is needed to have flashes of "no results found"
            if launch_app_by_id_visible.take() {
                if let Some(app_id) = text.as_ref().map(|t| t.parse::<u32>().ok()).flatten() {
                    launch_app_by_id_visible.set(true);
                    list_store.insert(
                        1,
                        &GSteamAppObject::new(AppModel {
                            app_id,
                            app_name: format!("App {app_id}"),
                            app_type: AppModelType::App,
                            developer: "Unknown".to_string(),
                            image_url: None,
                            metacritic_score: None,
                        }),
                    );
                }

                app_achievement_string_filter.set_search(text.as_deref());
                app_stat_string_filter.set_search(text.as_deref());
                list_string_filter.set_search(text.as_deref());
                list_store.remove(0);
                return;
            }

            if let Some(app_id) = text.clone().map(|t| t.parse::<u32>().ok()).flatten() {
                launch_app_by_id_visible.set(true);
                list_store.insert(
                    0,
                    &GSteamAppObject::new(AppModel {
                        app_id,
                        app_name: format!("App {app_id}"),
                        app_type: AppModelType::App,
                        developer: "Unknown".to_string(),
                        image_url: None,
                        metacritic_score: None,
                    }),
                );
            }

            app_achievement_string_filter.set_search(text.as_deref());
            app_stat_string_filter.set_search(text.as_deref());
            list_string_filter.set_search(text.as_deref());
        }
    ));

    list_filter_model.connect_items_changed(clone!(
        #[weak]
        list_of_apps_or_no_result,
        move |model, _, _, _| {
            if model.n_items() == 0 {
                list_of_apps_or_no_result.set_visible_child_name("empty");
            } else {
                list_of_apps_or_no_result.set_visible_child_name("list");
            }
        }
    ));

    // Back button handler
    back_button.connect_clicked(clone!(
        #[weak]
        list_stack,
        #[weak]
        app_id,
        #[weak]
        menu_model,
        #[weak]
        application,
        #[weak]
        app_achievements_model,
        #[weak]
        app_stat_model,
        #[strong]
        cancel_timed_unlock,
        move |_| {
            cancel_timed_unlock.store(true, std::sync::atomic::Ordering::Relaxed);
            list_stack.set_visible_child_name("list");
            set_context_popover_to_app_list_context(&menu_model, &application);
            if let Some(app_id) = app_id.take() {
                spawn_blocking(move || {
                    let _ = StopApp { app_id }.request();
                });
            }

            // Clear achievements and stats for performance, but wait a bit before doing so
            // to avoid flashes of the data disappearing during the animation
            let handle = spawn_blocking(move || {
                std::thread::sleep(std::time::Duration::from_millis(500));
            });

            MainContext::default().spawn_local(async move {
                if Some(()) != handle.await.ok() {
                    eprintln!("[CLIENT] Threading task failed");
                }

                app_achievements_model.remove_all();
                app_stat_model.remove_all();
            });
        }
    ));

    // App actions
    let action_refresh_app_list = SimpleAction::new("refresh_app_list", None);
    action_refresh_app_list.connect_activate(clone!(
        #[strong]
        list_view,
        #[strong]
        list_store,
        #[weak]
        list_scrolled_window,
        #[weak]
        list_of_apps_or_no_result,
        #[weak]
        app_list_no_result_label,
        #[weak]
        list_stack,
        #[weak]
        search_entry,
        move |_, _| {
            list_stack.set_visible_child_name("loading");
            search_entry.set_sensitive(false);
            let apps = spawn_blocking(move || GetOwnedAppList.request());
            MainContext::default().spawn_local(clone!(
                #[weak]
                list_view,
                #[weak]
                list_scrolled_window,
                #[weak]
                list_of_apps_or_no_result,
                #[weak]
                app_list_no_result_label,
                #[weak]
                list_store,
                #[weak]
                list_stack,
                #[weak]
                search_entry,
                async move {
                    match apps.await {
                        Ok(Ok(app_vec)) => {
                            search_entry.set_sensitive(true);

                            if app_vec.is_empty() {
                                app_list_no_result_label.set_text("No apps found on your account. Search for App Id to get started.");
                                list_of_apps_or_no_result.set_visible_child_name("empty");
                                list_scrolled_window.set_child(Some(&list_view));
                                list_stack.set_visible_child_name("list");
                            } else {
                                list_store.remove_all();
                                let mut models: Vec<GSteamAppObject> =
                                    app_vec.into_iter().map(GSteamAppObject::new).collect();
                                models.sort_by(|a, b| {
                                    let a_name = a.property::<String>("app_name");
                                    let b_name = b.property::<String>("app_name");
                                    a_name.to_lowercase().cmp(&b_name.to_lowercase())
                                });
                                list_store.extend_from_slice(&models);
                                list_scrolled_window.set_child(Some(&list_view));
                                list_stack.set_visible_child_name("list");
                                app_list_no_result_label.set_text("No results. Check for spelling mistakes or try typing an App Id.");
                            }
                        },
                        Ok(Err(sam_error)) if sam_error == SamError::AppListRetrievalFailed => {
                            search_entry.set_sensitive(true);
                            app_list_no_result_label.set_text("Failed to load library. Check your internet connection. Search for App Id to get started.");
                            list_of_apps_or_no_result.set_visible_child_name("empty");
                            list_scrolled_window.set_child(Some(&list_view));
                            list_stack.set_visible_child_name("list");
                        },
                        Ok(Err(sam_error)) => {
                            eprintln!("[CLIENT] Unknown error: {}", sam_error);
                            let label = Label::new(Some("SamRewritten could not connect to Steam. Is it running?"));
                            list_scrolled_window.set_child(Some(&label));
                            list_stack.set_visible_child_name("list");
                        }
                        Err(join_error) => {
                            eprintln!("Spawn blocking error: {:?}", join_error);
                        }
                    };
                }
            ));
        }
    ));

    let action_refresh_achievements_list = SimpleAction::new("refresh_achievements_list", None);
    action_refresh_achievements_list.set_enabled(false);
    action_refresh_achievements_list.connect_activate(clone!(
        #[strong]
        app_id,
        #[strong]
        app_unlocked_achievements_count,
        #[weak]
        application,
        #[weak]
        app_achievements_model,
        #[weak]
        app_stat_model,
        #[weak]
        app_achievement_count_value,
        #[weak]
        app_stats_count_value,
        #[weak]
        app_stack,
        #[weak]
        achievements_manual_adjustment,
        #[weak]
        achievements_manual_start,
        #[weak]
        app_achievements_stack,
        #[strong]
        cancel_timed_unlock,
        move |_, _| {
            app_stack.set_visible_child_name("loading");
            set_app_action_enabled(&application, "refresh_achievements_list", false);
            app_achievements_model.remove_all();
            app_stat_model.remove_all();
            cancel_timed_unlock.store(true, std::sync::atomic::Ordering::Relaxed);
            app_achievements_stack.set_visible_child_name("manual");

            let app_id_copy = app_id.get().unwrap();
            let handle = spawn_blocking(move || {
                let achievements = GetAchievements {
                    app_id: app_id_copy,
                }
                .request();
                let stats = GetStats {
                    app_id: app_id_copy,
                }
                .request();
                (achievements, stats)
            });

            MainContext::default().spawn_local(clone!(
                #[strong]
                app_unlocked_achievements_count,
                async move {
                    let Ok((Ok(achievements), Ok(stats))) = handle.await else {
                        return app_stack.set_visible_child_name("failed");
                    };

                    let achievement_len = achievements.len();
                    let achievement_unlocked_len =
                        achievements.iter().filter(|ach| ach.is_achieved).count();
                    app_unlocked_achievements_count.set(achievement_unlocked_len);

                    app_stats_count_value.set_label(&format!("{}", stats.len()));
                    app_achievement_count_value
                        .set_label(&format!("{achievement_unlocked_len} / {achievement_len}"));

                    let objects: Vec<GAchievementObject> = achievements
                        .into_iter()
                        .map(GAchievementObject::new)
                        .collect();
                    app_achievements_model.extend_from_slice(&objects);

                    let objects: Vec<GStatObject> =
                        stats.into_iter().map(GStatObject::new).collect();
                    app_stat_model.extend_from_slice(&objects);

                    if achievement_len > 0 {
                        app_stack.set_visible_child_name("achievements");
                    } else {
                        app_stack.set_visible_child_name("empty");
                    }

                    achievements_manual_start
                        .set_sensitive(achievement_unlocked_len != achievement_len);

                    let lower = std::cmp::min(achievement_unlocked_len + 1, achievement_len);
                    achievements_manual_adjustment.set_lower(lower as f64);
                    achievements_manual_adjustment.set_upper(achievement_len as f64);
                    achievements_manual_adjustment.set_value(achievement_len as f64);

                    set_app_action_enabled(&application, "refresh_achievements_list", true);
                    set_app_action_enabled(&application, "clear_all_stats_and_achievements", true);
                }
            ));
        }
    ));

    let action_clear_all_stats_and_achievements =
        SimpleAction::new("clear_all_stats_and_achievements", None);
    action_clear_all_stats_and_achievements.set_enabled(false);
    action_clear_all_stats_and_achievements.connect_activate(clone!(
        #[strong]
        app_id,
        #[weak]
        application,
        #[weak]
        app_achievements_model,
        #[weak]
        app_stat_model,
        #[weak]
        action_refresh_achievements_list,
        #[weak]
        app_stack,
        move |_, _| {
            app_stack.set_visible_child_name("loading");
            set_app_action_enabled(&application, "clear_all_stats_and_achievements", false);
            app_achievements_model.remove_all();
            app_stat_model.remove_all();

            let app_id_copy = app_id.get().unwrap();
            let handle = spawn_blocking(move || {
                let success = ResetStats {
                    app_id: app_id_copy,
                    achievements_too: true,
                }
                .request();
                success
            });

            MainContext::default().spawn_local(clone!(async move {
                let Ok(Ok(_success)) = handle.await else {
                    return app_stack.set_visible_child_name("failed");
                };

                action_refresh_achievements_list.activate(None);
            }));
        }
    ));

    list_stack.connect_visible_child_notify(clone!(
        #[weak]
        back_button,
        #[weak]
        application,
        #[weak]
        app_stack,
        #[weak]
        search_entry,
        #[weak]
        action_refresh_app_list,
        move |stack| {
            if stack.visible_child_name().as_deref() == Some("loading") {
                back_button.set_sensitive(false);
                action_refresh_app_list.set_enabled(false);
            } else if stack.visible_child_name().as_deref() == Some("app") {
                search_entry.set_text("");
                search_entry.set_placeholder_text(Some("Achievement or stat..."));
                back_button.set_sensitive(true);
                action_refresh_app_list.set_enabled(false);
            } else {
                search_entry.set_text("");
                search_entry.set_placeholder_text(Some("App name..."));
                back_button.set_sensitive(false);
                action_refresh_app_list.set_enabled(true);

                let auto_launch_app = gui_args.auto_open.get();
                if auto_launch_app > 0 {
                    gui_args.auto_open.set(0);

                    // let mut found_iter = None;
                    for ach in &list_store {
                        if let Ok(obj) = ach {
                            let g_app = obj
                                .downcast::<GSteamAppObject>()
                                .expect("Not a GSteamAppObject");
                            if g_app.app_id() == auto_launch_app {
                                // found_iter = Some(g_app);
                                switch_from_app_list_to_app(
                                    &g_app,
                                    application.clone(),
                                    &app_type_value,
                                    &app_developer_value,
                                    &app_achievement_count_value,
                                    &app_stats_count_value,
                                    app_stack.clone(),
                                    &app_id,
                                    &app_metacritic_box,
                                    &app_metacritic_value,
                                    &app_shimmer_image,
                                    &app_label,
                                    &menu_model,
                                    &stack,
                                );
                                break;
                            }
                        }
                    }
                }
            }
        }
    ));

    app_stack.set_visible_child_name("loading");
    list_stack.set_visible_child_name("loading");
    action_refresh_app_list.activate(None);
    action_refresh_app_list.set_enabled(false);

    setup_app_actions(
        application,
        &about_dialog,
        &action_refresh_app_list,
        &action_refresh_achievements_list,
        &action_clear_all_stats_and_achievements,
    );

    window.present();

    ExitCode::SUCCESS
}
