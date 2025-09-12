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

use super::request::{Request, SetFloatStat, SetIntStat};
use super::stat::GStatObject;
use glib::prelude::ToValue;
use gtk::{
    Adjustment, Align, Box, ClosureExpression, FilterListModel, Frame, Label, ListItem, ListView,
    NoSelection, Orientation, ScrolledWindow, SignalListItemFactory, SpinButton, StringFilter,
    StringFilterMatchMode, Widget,
    gio::{ListStore, spawn_blocking},
    glib::{self, SignalHandlerId, object::Cast, translate::FromGlib},
    pango::EllipsizeMode,
    prelude::*,
};
use std::{cell::RefCell, ffi::c_ulong, sync::mpsc::channel, time::Duration};
use log;

/// Create the stats view, including model, filter, and UI.
pub fn create_stats_view() -> (Frame, ListStore, StringFilter) {
    let stats_list_factory = SignalListItemFactory::new();
    let app_stats_model = ListStore::new::<GStatObject>();

    let app_stats_string_filter = StringFilter::builder()
        .expression(&GStatObject::this_expression("display-name"))
        .match_mode(StringFilterMatchMode::Substring)
        .ignore_case(true)
        .build();
    let app_stats_filter_model = FilterListModel::builder()
        .model(&app_stats_model)
        .filter(&app_stats_string_filter)
        .build();
    let app_stats_selection_model = NoSelection::new(Some(app_stats_filter_model.clone()));

    let app_stats_list_view = ListView::builder()
        .orientation(Orientation::Vertical)
        .model(&app_stats_selection_model)
        .factory(&stats_list_factory)
        .build();
    let app_stats_scrolled_window = ScrolledWindow::builder()
        .child(&app_stats_list_view)
        .vexpand(true)
        .build();

    stats_list_factory.connect_setup(move |_, list_item| {
        let adjustment = Adjustment::builder()
            .lower(i32::MIN as f64)
            .upper(i32::MAX as f64)
            .page_size(0.0)
            .build();

        let spin_button = SpinButton::builder().adjustment(&adjustment).build();

        let button_box = Box::builder()
            .orientation(Orientation::Vertical)
            .halign(Align::End)
            .build();
        button_box.append(&spin_button);
        let spacer = Box::builder()
            .orientation(Orientation::Horizontal)
            .hexpand(true)
            .build();
        let name_label = Label::builder()
            .ellipsize(EllipsizeMode::End)
            .halign(Align::Start)
            .build();

        let stat_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .margin_top(8)
            .margin_bottom(8)
            .margin_start(8)
            .margin_end(8)
            .build();
        stat_box.append(&name_label);
        stat_box.append(&spacer);

        let icon_increment_only = gtk::Image::from_icon_name("go-up-symbolic");
        icon_increment_only.set_margin_end(8);
        icon_increment_only.set_tooltip_text(Some("Increment only"));
        stat_box.append(&icon_increment_only);

        let protected_icon = gtk::Image::from_icon_name("action-unavailable-symbolic");
        protected_icon.set_margin_end(8);
        protected_icon.set_tooltip_text(Some("This statistic is protected."));
        stat_box.append(&protected_icon);

        stat_box.append(&button_box);
        if let Some(list_item) = list_item.downcast_ref::<gtk::ListItem>() {
            list_item.set_child(Some(&stat_box));

            // Property expressions and bindings
            list_item
                .property_expression("item")
                .chain_property::<GStatObject>("display-name")
                .bind(&name_label, "label", Widget::NONE);

            list_item
                .property_expression("item")
                .chain_property::<GStatObject>("current-value")
                .bind(&adjustment, "value", Widget::NONE);

            list_item
                .property_expression("item")
                .chain_property::<GStatObject>("is-increment-only")
                .bind(&icon_increment_only, "visible", Widget::NONE);

            // Custom expressions
            let is_integer_expr = list_item
                .property_expression("item")
                .chain_property::<GStatObject>("is-integer");
            let is_integer_expr_2 = is_integer_expr.clone();
            let is_increment_only_expr = list_item
                .property_expression("item")
                .chain_property::<GStatObject>("is-increment-only");
            let original_value_expr = list_item
                .property_expression("item")
                .chain_property::<GStatObject>("original-value");
            let permission_expr = list_item
                .property_expression("item")
                .chain_property::<GStatObject>("permission");
            let permission_expr_2 = permission_expr.clone();

            let adjustment_step_increment_closure = glib::RustClosure::new(|values: &[glib::Value]| {
                let is_integer = values.get(1).and_then(|val| val.get::<bool>().ok()).unwrap_or(false);
                let step_increment = if is_integer { 1.0 } else { 0.01 };
                Some(step_increment.to_value())
            });

            let adjustment_lower_closure = glib::RustClosure::new(|values: &[glib::Value]| {
                let original_value = values.get(1).and_then(|val| val.get::<f64>().ok()).unwrap_or(0f64);
                let is_increment_only = values.get(2).and_then(|val| val.get::<bool>().ok()).unwrap_or(false);
                let lower = if is_increment_only {
                    original_value
                } else {
                    i32::MIN as f64
                };
                Some(lower.to_value())
            });

            let spin_button_digits_closure = glib::RustClosure::new(|values: &[glib::Value]| {
                let is_integer = values.get(1).and_then(|val| val.get::<bool>().ok()).unwrap_or(false);
                let digits: u32 = if is_integer { 0 } else { 2 };
                Some(digits.to_value())
            });

            let permission_sensitive_closure = glib::RustClosure::new(|values: &[glib::Value]| {
                let permission = values.get(1).and_then(|val| val.get::<i32>().ok()).unwrap_or(0);
                let is_sensitive = (permission & 2) == 0;
                Some(is_sensitive.to_value())
            });

            let permission_protected_closure = glib::RustClosure::new(|values: &[glib::Value]| {
                let permission = values.get(1).and_then(|val| val.get::<i32>().ok()).unwrap_or(0);
                let is_protected = (permission & 2) != 0;
                Some(is_protected.to_value())
            });

            let adjustment_step_increment_expression =
                ClosureExpression::new::<f64>(&[is_integer_expr], adjustment_step_increment_closure);
            adjustment_step_increment_expression.bind(&adjustment, "step-increment", Widget::NONE);

            let adjustment_lower_expression = ClosureExpression::new::<f64>(
                &[original_value_expr, is_increment_only_expr],
                adjustment_lower_closure,
            );
            adjustment_lower_expression.bind(&adjustment, "lower", Widget::NONE);

            let spin_button_digits_expression =
                ClosureExpression::new::<u32>(&[is_integer_expr_2], spin_button_digits_closure);
            spin_button_digits_expression.bind(&spin_button, "digits", Widget::NONE);

            let permission_sensitive_expr =
                ClosureExpression::new::<bool>(&[permission_expr], permission_sensitive_closure);
            permission_sensitive_expr.bind(&spin_button, "sensitive", Widget::NONE);

            let permission_protected_expr =
                ClosureExpression::new::<bool>(&[permission_expr_2], permission_protected_closure);
            permission_protected_expr.bind(&protected_icon, "visible", Widget::NONE);
        } else {
            log::error!("list_item was not a ListItem; skipping child set");
        }
    });

    stats_list_factory.connect_bind(move |_, list_item| unsafe {
        let list_item = match list_item.downcast_ref::<ListItem>() {
            Some(li) => li,
            _ => {
                log::error!("ListItem cast failed in bind");
                return;
            }
        };
        let stat_object = match list_item
            .item()
            .and_then(|item| item.downcast::<GStatObject>().ok()) {
            Some(so) => so,
            _ => {
                log::error!("Item was not a GStatObject");
                return;
            }
        };
        // Optimized: direct traversal, no unnecessary clones
        let spin_button = list_item
            .child()
            .and_then(|child| child.downcast::<Box>().ok())
            .and_then(|stat_box| stat_box.last_child())
            .and_then(|button_box| button_box.downcast::<Box>().ok())
            .and_then(|button_box| button_box.last_child())
            .and_then(|spin_button_widget| spin_button_widget.downcast::<SpinButton>().ok());
        let Some(spin_button) = spin_button else {
            log::error!("Could not find SpinButton widget");
            return;
        };

        // Use a single sender per bind, avoid unnecessary channel recreation
        let (sender, _) = channel::<f64>();
        let sender = RefCell::new(sender);

        let handler_id = spin_button.connect_value_changed({
            let stat_object = stat_object.clone();
            move |button| {
                let val = button.value();
                if sender.borrow_mut().send(val).is_ok() {
                    return;
                }
                // Only recreate channel if send fails
                let (new_sender, new_receiver) = channel();
                *sender.borrow_mut() = new_sender;
                let mut value = val;
                let integer_stat = stat_object.is_integer();
                let stat_id = stat_object.id().clone();
                let stat_object_clone = stat_object.clone();
                let app_id = stat_object.app_id().clone();

                glib::spawn_future_local(async move {
                    let join_handle = spawn_blocking(move || {
                        while let Ok(new) = new_receiver.recv_timeout(Duration::from_millis(500)) {
                            value = (new * 100.0).round() / 100.0;
                        }
                        let res = if integer_stat {
                            SetIntStat {
                                app_id,
                                stat_id,
                                value: value.round() as i32,
                            }
                            .request()
                        } else {
                            SetFloatStat {
                                app_id,
                                stat_id,
                                value: value as f32,
                            }
                            .request()
                        };
                        match res {
                            Ok(success) if success => (true, value),
                            _ => (false, value),
                        }
                    });
                    let (success, debounced_value) =
                        match join_handle.await {
                            Ok((success, debounced_value)) => (success, debounced_value),
                            Err(e) => {
                                log::error!("spawn_blocking task panicked: {:?}", e);
                                (false, value)
                            }
                        };
                    if success {
                        stat_object_clone.set_original_value(debounced_value);
                    } else {
                        stat_object_clone.set_current_value(stat_object_clone.original_value());
                    }
                });
            }
        });
        spin_button.set_data("handler", handler_id.as_raw());
    });

    stats_list_factory.connect_unbind(move |_, list_item| unsafe {
        let list_item = match list_item.downcast_ref::<ListItem>() {
            Some(li) => li,
            _ => {
                log::error!("ListItem cast failed in unbind");
                return;
            }
        };
        let spin_button = list_item
            .child()
            .and_then(|child| child.downcast::<Box>().ok())
            .and_then(|stat_box| stat_box.last_child())
            .and_then(|button_box| button_box.downcast::<Box>().ok())
            .and_then(|button_box| button_box.last_child())
            .and_then(|spin_button_widget| spin_button_widget.downcast::<SpinButton>().ok());
        let Some(spin_button) = spin_button else {
            log::error!("Could not find SpinButton widget");
            return;
        };
        // Disconnect previous handler if it exists
        if let Some(handler_id) = spin_button.data("handler") {
            let ulong: c_ulong = *handler_id.as_ptr();
            let signal_handler = SignalHandlerId::from_glib(ulong);
            spin_button.disconnect(signal_handler);
        } else {
            eprintln!("[CLIENT] Stat spinbox unbind failed");
        }
    });

    let app_stats_frame = Frame::builder()
        .margin_end(15)
        .margin_start(15)
        .margin_top(15)
        .margin_bottom(15)
        .child(&app_stats_scrolled_window)
        .build();

    (app_stats_frame, app_stats_model, app_stats_string_filter)
}
