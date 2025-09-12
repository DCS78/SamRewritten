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
use gtk::glib::subclass::types::ObjectSubclassIsExt;

glib::wrapper! {
    pub struct ShimmerImage(ObjectSubclass<imp::ShimmerImage>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ShimmerImage {
    /// Create a new ShimmerImage widget.
    pub fn new() -> Self {
        glib::Object::builder()
            .property("url", None::<String>)
            .build()
    }

    /// Reset the image state and mark as failed.
    pub fn reset(&self) {
        self.imp().url.borrow_mut().take();
        self.imp().texture.borrow_mut().take();
        self.imp().receiver.borrow_mut().take();
        self.imp().loaded.borrow_mut().take();
        self.imp().failed.set(true);
    }
}

mod imp {
    use crate::dev_println;
    use glib::Properties;
    use gtk::gdk::{RGBA, Texture};
    use gtk::gio::spawn_blocking;
    use gtk::glib::{self, Bytes, base64_encode};
    use gtk::graphene::{Point, Rect, Size};
    use gtk::gsk::{ColorStop, LinearGradientNode, RoundedRect};
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use reqwest::blocking::Client;
    use std::{
        cell::{Cell, RefCell},
        env::temp_dir,
        fs::{exists, write},
        sync::mpsc::{Receiver, TryRecvError, sync_channel},
    };

    const GRADIENT_WIDTH: f32 = 0.8;
    const BASE_COLOR: RGBA = RGBA::new(0.7, 0.7, 0.7, 1.0);
    const HIGHLIGHT_COLOR: RGBA = RGBA::new(0.8, 0.8, 0.8, 1.0);

    /// Internal implementation of ShimmerImage properties and state.
    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::ShimmerImage)]
    pub struct ShimmerImage {
        #[property(get, set)]
        pub image_width: Cell<i32>,
        #[property(get, set)]
        pub image_height: Cell<i32>,
        pub start: Cell<i64>,
        pub current: Cell<i64>,
        #[property(get, set)]
        pub url: RefCell<Option<String>>,
        #[property(get, set)]
        pub loaded: RefCell<Option<String>>,
        pub failed: Cell<bool>,
        pub receiver: RefCell<Option<Receiver<Texture>>>,
        pub texture: RefCell<Option<Texture>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ShimmerImage {
        const NAME: &'static str = "ShimmerImage";
        type Type = super::ShimmerImage;
        type ParentType = gtk::Widget;
    }

    #[glib::derived_properties]
    impl ObjectImpl for ShimmerImage {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.reset();

            obj.set_size_request(231, 87);
            obj.add_tick_callback(|widget, clock| {
                if let Some(this) = widget.downcast_ref::<super::ShimmerImage>() {
                    //Enabling this will cause some of the images to retain their old texture
                    //even if the url property changes, but only if the widget was rendered before
                    //and then jumps into view while it's contents are still cached.
                    //if this.imp().texture.borrow().is_none() {
                    this.queue_draw();
                    //}

                    let imp = this.imp();
                    imp.current.set(clock.frame_time());
                    if imp.start.get() == 0 {
                        imp.start.set(clock.frame_time());
                    }
                }
                glib::ControlFlow::Continue
            });
        }
    }

    impl WidgetImpl for ShimmerImage {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let widget = self.obj();
            let width = widget.width() as f32;
            let height = widget.height() as f32;

            let rect = Rect::new(0.0, 0.0, width, height);
            let size = Size::new(5.0, 5.0);
            let rounded = RoundedRect::new(rect, size, size, size, size);
            snapshot.push_rounded_clip(&rounded);

            // Only take the url if it is Some and not already loaded
            let url_opt = self.url.borrow_mut();
            if let Some(url) = url_opt.as_ref() {
                if Some(url.as_str()) != self.loaded.borrow().as_deref() {
                    self.texture.borrow_mut().take();
                    self.loaded.borrow_mut().take();
                    self.receiver.borrow_mut().take();
                    self.load(url.as_str());
                    self.loaded.borrow_mut().replace(url.clone());
                }
            }

            // Only take the receiver if it is Some
            let mut receiver_opt = self.receiver.borrow_mut();
            if let Some(receiver) = receiver_opt.as_mut() {
                match receiver.try_recv() {
                    Ok(texture) => {
                        self.texture.borrow_mut().replace(texture);
                        receiver_opt.take();
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => {
                        self.failed.set(true);
                        receiver_opt.take();
                    }
                }
            }

            if self.failed.get() {
                // TODO: Insert an icon in the middle: insert-image-symbolic
                snapshot.append_color(&BASE_COLOR, &rect);
            } else if let Some(texture) = &*self.texture.borrow() {
                snapshot.append_texture(texture, &rect);
            } else {
                // convert from continuous microseconds to relative seconds
                let progress =
                    ((self.current.get() - self.start.get()) / 1000) as f32 / 1000.0 % 1.0;
                let progress = ease_in_out(progress);
                let start_pos = -GRADIENT_WIDTH + (1.0 + 2.0 * GRADIENT_WIDTH) * progress;
                let end_pos = start_pos + GRADIENT_WIDTH;

                let color_stops = vec![
                    ColorStop::new(0.0, BASE_COLOR),
                    ColorStop::new(0.3, HIGHLIGHT_COLOR),
                    ColorStop::new(0.5, HIGHLIGHT_COLOR),
                    ColorStop::new(0.7, HIGHLIGHT_COLOR),
                    ColorStop::new(1.0, BASE_COLOR),
                ];

                let gradient = LinearGradientNode::new(
                    &rect,
                    &Point::new(width * start_pos, 0.0),
                    &Point::new(width * end_pos, 0.0),
                    color_stops.as_slice(),
                );
                snapshot.append_node(&gradient);
            }

            fn ease_in_out(t: f32) -> f32 {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }

            snapshot.pop();
        }
    }

    impl ShimmerImage {
        fn load(&self, url: &str) {
            self.failed.set(false);

            let mut split = url.splitn(2, "://");
            let scheme = split.next();
            let rest = split.next();
            let (scheme, rest) = match (scheme, rest) {
                (Some(scheme), Some(rest)) => (scheme, rest),
                _ => {
                    dev_println!("[CLIENT] Invalid URL: {url}");
                    self.failed.set(true);
                    return;
                }
            };

            let (sender, receiver) = sync_channel::<Texture>(0);
            self.receiver.borrow_mut().replace(receiver);
            let failed = self.failed.clone();

            match scheme {
                "https" => {
                    let mut path = temp_dir();
                    let url = url.to_string();
                    path.push(format!("{}.jpg", base64_encode(url.as_bytes())));

                    spawn_blocking(move || {
                        let exists_result = std::panic::catch_unwind(|| exists(path.as_path()));
                        let exists: Result<bool, std::io::Error> = match exists_result {
                            Ok(val) => val,
                            Err(_) => {
                                dev_println!("[CLIENT] Failed to check if file exists for {url}");
                                Ok(false)
                            }
                        };
                        match exists {
                            Ok(false) => {
                                dev_println!("[CLIENT] Downloading: {url}");
                                //Download and store to path
                                let response = match Client::new()
                                    .get(url.as_str())
                                    .send()
                                    .and_then(|response| response.error_for_status())
                                    .and_then(|response| response.bytes())
                                {
                                    Ok(response) => response,
                                    Err(error) => {
                                        failed.set(true);
                                        return eprintln!("[CLIENT] Failed to download {url}: {error}");
                                    }
                                };

                                if let Err(error) = write(path.as_path(), response) {
                                    failed.set(true);
                                    eprintln!("[CLIENT] Failed to write {url} to {path:?}: {error}");
                                    return;
                                }
                            }
                            Ok(true) => {
                                dev_println!("[CLIENT] Cached loading: {url}");
                            }
                            Err(e) => {
                                failed.set(true);
                                eprintln!("[CLIENT] Error checking if file exists for {url}: {e}");
                                return;
                            }
                        }

                        let data = match std::fs::read(path.as_path()) {
                            Ok(data) => data,
                            Err(error) => {
                                failed.set(true);
                                eprintln!("[CLIENT] Failed to read {url} from {path:?}: {error}");
                                return;
                            }
                        };

                        match Texture::from_bytes(&Bytes::from(data.as_slice())) {
                            Ok(texture) => {
                                sender.send(texture).ok();
                            }
                            Err(error) => {
                                failed.set(true);
                                eprintln!("[CLIENT] Failed to create {url} from bytes: {error}");
                            }
                        }
                    });
                }
                "file" => {
                    let file_path = rest.to_string();
                    spawn_blocking(move || {
                        let data = match std::fs::read(&file_path) {
                            Ok(data) => data,
                            Err(error) => {
                                failed.set(true);
                                eprintln!("[CLIENT] Failed to read {file_path}: {error}");
                                return;
                            }
                        };

                        match Texture::from_bytes(&Bytes::from(data.as_slice())) {
                            Ok(texture) => {
                                sender.send(texture).ok();
                            }
                            Err(error) => {
                                failed.set(true);
                                eprintln!(
                                    "[CLIENT] Failed to create {file_path} from bytes: {error}"
                                );
                            }
                        }
                    });
                }
                _ => {
                    failed.set(true);
                    dev_println!("[CLIENT] Unsupported URL scheme: {url}");
                }
            }
        }
    }
}
