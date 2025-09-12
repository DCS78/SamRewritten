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

use gtk::glib;

glib::wrapper! {
    pub struct CustomProgressBar(ObjectSubclass<imp::CustomProgressBar>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl CustomProgressBar {
    /// Create a new CustomProgressBar with value initialized to 0.
    pub fn new() -> Self {
        glib::Object::builder().property("value", 0f32).build()
    }
}

mod imp {
    use glib::Properties;
    use gtk::gdk::RGBA;
    use gtk::glib::{self};
    use gtk::graphene::Rect;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use std::cell::Cell;

    // TODO: If building with Adwaita, use the platform accent color
    const BAR_COLOR: RGBA = RGBA::new(0.6, 0.6, 0.9, 0.2);

    /// Internal implementation of CustomProgressBar properties.
    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::CustomProgressBar)]
    pub struct CustomProgressBar {
        #[property(get, set)]
        pub value: Cell<f32>, // Value from 0 to 100
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CustomProgressBar {
        const NAME: &'static str = "CustomProgressBar";
        type Type = super::CustomProgressBar;
        type ParentType = gtk::Widget;
    }

    #[glib::derived_properties]
    impl ObjectImpl for CustomProgressBar {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.set_size_request(200, 20); // Set a default size for the progress bar
        }
    }

    impl WidgetImpl for CustomProgressBar {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let widget = self.obj();
            let width = widget.width() as f32;
            let height = widget.height() as f32;
            // Use f32::clamp for clarity and possible inlining
            let value = f32::clamp(self.value.get(), 0.0, 100.0);
            if value > 0.0 && width > 0.0 && height > 0.0 {
                let progress_width = width * (value / 100.0);
                if progress_width > 0.0 {
                    let progress_rect = Rect::new(0.0, 0.0, progress_width, height);
                    snapshot.append_color(&BAR_COLOR, &progress_rect);
                }
            }
        }
    }
}
