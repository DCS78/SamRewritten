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


//! Provides a safe Rust abstraction over the `ISteamApps001` FFI interface.
//! This module allows safe access to Steam application data via the Steamworks API.
use crate::steam_client::steam_apps_001_vtable::ISteamApps001;
use crate::steam_client::wrapper_types::SteamClientError;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::sync::Arc;

/// Safe wrapper for the `ISteamApps001` interface.
#[derive(Debug, Clone)]
pub struct SteamApps001 {
    inner: Arc<SteamApps001Inner>,
}

#[derive(Debug)]
struct SteamApps001Inner {
    ptr: *mut ISteamApps001,
}

/// Enum representing possible app data keys for SteamApps001 queries.
#[derive(Debug, Clone)]
pub enum SteamApps001AppDataKeys<'a> {
    /// The app's name.
    Name,
    /// The app's logo.
    Logo,
    /// The app's small capsule image for a given language.
    SmallCapsule(&'a str),
    /// The app's Metacritic score.
    MetacriticScore,
    /// The app's developer.
    Developer,
}

impl<'a> SteamApps001AppDataKeys<'a> {
    /// Returns the string key as expected by the Steam API, including null terminator.
    pub fn as_string(&self) -> String {
        match self {
            SteamApps001AppDataKeys::Name => "name\0".to_string(),
            SteamApps001AppDataKeys::SmallCapsule(language) => {
                format!("small_capsule/{language}\0")
            }
            SteamApps001AppDataKeys::Logo => "logo\0".to_string(),
            SteamApps001AppDataKeys::MetacriticScore => "metacritic_score\0".to_string(),
            SteamApps001AppDataKeys::Developer => "developer\0".to_string(),
        }
    }
}

impl SteamApps001 {
    /// Creates a new `SteamApps001` instance from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be valid and remain valid for the lifetime of the `SteamApps001` instance.
    pub unsafe fn from_raw(ptr: *mut ISteamApps001) -> Self {
        Self {
            inner: Arc::new(SteamApps001Inner { ptr }),
        }
    }

    /// Retrieves app data for a given app ID and key.
    ///
    /// # Arguments
    /// * `app_id` - The Steam application ID.
    /// * `key` - The key to query (should be null-terminated as expected by the Steam API).
    ///
    /// # Errors
    /// Returns `SteamClientError` if the vtable is null or the FFI call fails.
    pub fn get_app_data(&self, app_id: &u32, key: &str) -> Result<String, SteamClientError> {
        let mut buffer = vec![0u8; 256];

        unsafe {
            // Get the vtable - return error if null
            let vtable = (*self.inner.ptr)
                .vtable
                .as_ref()
                .ok_or(SteamClientError::NullVtable)?;

            // Call through the vtable
            let result = (vtable.get_app_data)(
                self.inner.ptr,
                *app_id,
                key.as_ptr() as *const c_char,
                buffer.as_mut_ptr() as *mut c_char,
                buffer.len() as c_int,
            );

            if result == 0 {
                return Err(SteamClientError::UnknownError);
            }

            let c_str = CStr::from_ptr(buffer.as_ptr() as *const c_char);
            Ok(c_str.to_string_lossy().into_owned())
        }
    }
}
