use gtk::glib;
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

use crate::frontend::MainApplication;
use crate::frontend::achievement::GAchievementObject;
use crate::frontend::achievement_automatic_view::create_achievements_automatic_view;
use crate::frontend::achievement_manual_view::create_achievements_manual_view;
use gtk::gio::ListStore;
use gtk::prelude::*;
use gtk::{
    Adjustment, Button, CustomSorter, FilterListModel, Label, NoSelection, SortListModel,
    SpinButton, Stack, StackTransitionType, StringFilter, StringFilterMatchMode,
};
use std::cell::Cell;
use std::cmp::Ordering;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

/// Create the achievements view stack, models, and controls.
pub fn create_achievements_view(
    app_id: Rc<Cell<Option<u32>>>,
    app_unlocked_achievements_count: Rc<Cell<usize>>,
    application: &MainApplication,
    app_achievement_count_value: &Label,
) -> (
    Stack,
    ListStore,
    StringFilter,
    Adjustment,
    SpinButton,
    Button,
    Arc<AtomicBool>,
) {
    let app_achievements_model = ListStore::new::<GAchievementObject>();
    let app_timed_achievements_model = ListStore::new::<GAchievementObject>();

    let app_achievement_string_filter = StringFilter::builder()
        .expression(&GAchievementObject::this_expression("search-text"))
        .match_mode(StringFilterMatchMode::Substring)
        .ignore_case(true)
        .build();

    let app_achievement_filter_model = FilterListModel::builder()
        .model(&app_achievements_model)
        .filter(&app_achievement_string_filter)
        .build();
    let app_achievement_timed_filter_model = FilterListModel::builder()
        .model(&app_timed_achievements_model)
        .filter(&app_achievement_string_filter)
        .build();

    let global_achieved_percent_sorter = CustomSorter::new(|obj1, obj2| {
        let achievement1 = match obj1.downcast_ref::<GAchievementObject>() {
            Some(a) => a,
            _ => {
                log::error!("obj1 is not a GAchievementObject in global_achieved_percent_sorter");
                return Ordering::Equal.into();
            }
        };
        let achievement2 = match obj2.downcast_ref::<GAchievementObject>() {
            Some(a) => a,
            _ => {
                log::error!("obj2 is not a GAchievementObject in global_achieved_percent_sorter");
                return Ordering::Equal.into();
            }
        };
        let percent1 = achievement1.global_achieved_percent();
        let percent2 = achievement2.global_achieved_percent();
        match percent2.partial_cmp(&percent1) {
            Some(ordering) => ordering.into(),
            none => {
                log::warn!("partial_cmp returned None in global_achieved_percent_sorter");
                Ordering::Equal.into()
            }
        }
    });
    let app_achievement_sort_model = SortListModel::builder()
        .model(&app_achievement_filter_model)
        .sorter(&global_achieved_percent_sorter)
        .build();

    let app_achievement_selection_model = NoSelection::new(Option::<ListStore>::None);
    app_achievement_selection_model.set_model(Some(&app_achievement_sort_model));
    let app_timed_achievement_selection_model = NoSelection::new(Option::<ListStore>::None);
    app_timed_achievement_selection_model.set_model(Some(&app_achievement_timed_filter_model));

    let achievement_views_stack = Stack::builder()
        .transition_type(StackTransitionType::SlideLeftRight)
        .build();

    let (
        achievements_manual_frame,
        achievements_manual_adjustment,
        achievements_manual_spinbox,
        achievements_manual_start,
        cancel_timed_unlock,
    ) = create_achievements_manual_view(
        &app_id,
        &app_unlocked_achievements_count,
        &app_achievement_selection_model,
        &app_achievements_model,
        &app_timed_achievements_model,
        &achievement_views_stack,
        &app_achievement_count_value,
        &application,
    );
    let (achievements_automatic_frame, _achievements_automatic_stop) =
        create_achievements_automatic_view(&app_timed_achievement_selection_model, &application);

    achievement_views_stack.add_named(&achievements_manual_frame, Some("manual"));
    achievement_views_stack.add_named(&achievements_automatic_frame, Some("automatic"));

    (
        achievement_views_stack,
        app_achievements_model,
        app_achievement_string_filter,
        achievements_manual_adjustment,
        achievements_manual_spinbox,
        achievements_manual_start,
        cancel_timed_unlock,
    )
}

/// Count the number of unlocked achievements in the given model.
pub fn count_unlocked_achievements(model: &ListStore) -> u32 {
    model
        .iter()
        .filter_map(|ach| {
            ach.ok().and_then(|obj: glib::Object| {
                let g_achievement = obj.downcast::<GAchievementObject>().ok()?;
                if g_achievement.is_achieved() {
                    Some(())
                } else {
                    None
                }
            })
        })
        .count() as u32
}
