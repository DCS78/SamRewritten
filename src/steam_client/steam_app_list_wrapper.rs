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

use crate::steam_client::steam_app_list_vtable::ISteamAppList;
use crate::steam_client::steamworks_types::AppId_t;
use crate::steam_client::wrapper_types::SteamClientError;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::sync::Arc;

/// Safe Rust wrapper for the ISteamAppList interface.
#[allow(dead_code)]
pub struct SteamAppList {
    inner: Arc<SteamAppListInner>,
}

#[allow(dead_code)]
struct SteamAppListInner {
    ptr: *mut ISteamAppList,
}

#[allow(dead_code)]
impl SteamAppList {
    /// Constructs a new `SteamAppList` from a raw pointer.
    /// # Safety
    /// The pointer must be valid and point to a live ISteamAppList.
    pub unsafe fn from_raw(ptr: *mut ISteamAppList) -> Self {
        Self {
            inner: Arc::new(SteamAppListInner { ptr }),
        }
    }

    /// Gets the name of an app by its AppId.
    pub fn get_app_name(&self, app_id: AppId_t) -> Result<String, SteamClientError> {
        let mut buffer = vec![0u8; 256];
        unsafe {
            let vtable = (*self.inner.ptr)
                .vtable
                .as_ref()
                .ok_or(SteamClientError::NullVtable)?;

            let result = (vtable.get_app_name)(
                self.inner.ptr,
                app_id,
                buffer.as_mut_ptr() as *mut c_char,
                buffer.len() as c_int,
            );

            match result {
                -1 => Err(SteamClientError::AppNotFound),
                len if len >= 0 => {
                    // Convert the null-terminated C string to a Rust string
                    let c_str = CStr::from_ptr(buffer.as_ptr() as *const c_char);
                    Ok(c_str.to_string_lossy().into_owned())
                }
                _ => Err(SteamClientError::UnknownError),
            }
        }
    }
}
