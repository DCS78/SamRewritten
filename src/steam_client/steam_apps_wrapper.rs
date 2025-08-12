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

//! Provides a safe Rust abstraction over the `ISteamApps` FFI interface.
use crate::steam_client::steam_apps_vtable::ISteamApps;
use crate::steam_client::wrapper_types::SteamClientError;
use std::sync::Arc;

/// Safe wrapper for the `ISteamApps` interface.
#[derive(Debug, Clone)]
pub struct SteamApps {
    inner: Arc<SteamAppsInner>,
}

#[derive(Debug)]
struct SteamAppsInner {
    ptr: *mut ISteamApps,
}

impl SteamApps {
    /// Creates a new `SteamApps` instance from a raw pointer.
    /// The pointer must be valid and remain valid for the lifetime of the `SteamApps` instance.
    pub unsafe fn from_raw(ptr: *mut ISteamApps) -> Self {
        Self {
            inner: Arc::new(SteamAppsInner { ptr }),
        }
    }

    /// Returns the current game language as a UTF-8 string.
    /// Panics if the vtable pointer is null.
    pub fn get_current_game_language(&self) -> String {
        unsafe {
            let vtable = (*self.inner.ptr)
                .vtable
                .as_ref()
                .expect("Null ISteamApps vtable");
            let lang_ptr = (vtable.get_current_game_language)(self.inner.ptr);
            std::ffi::CStr::from_ptr(lang_ptr)
                .to_string_lossy()
                .into_owned()
        }
    }

    /// Returns whether the user is subscribed to the given app ID.
    /// Returns `SteamClientError` if the vtable is null.
    pub fn is_subscribed_app(&self, app_id: u32) -> Result<bool, SteamClientError> {
        unsafe {
            // Get the vtable - return error if null
            let vtable = (*self.inner.ptr)
                .vtable
                .as_ref()
                .ok_or(SteamClientError::NullVtable)?;

            // Call through the vtable
            let is_subscribed = (vtable.b_is_subscribed_app)(self.inner.ptr, app_id);

            Ok(is_subscribed)
        }
    }
}
