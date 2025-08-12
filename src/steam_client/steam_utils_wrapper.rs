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

//! Provides a safe Rust abstraction over the `ISteamUtils` FFI interface.
use crate::dev_println;
use crate::steam_client::steam_utils_vtable::ISteamUtils;
use crate::steam_client::steamworks_types::{
    AppId_t, GlobalAchievementPercentagesReady_t, SteamAPICall_t,
};
use crate::steam_client::wrapper_types::{SteamCallbackId, SteamClientError};
use std::ffi::{c_int, c_void};
use std::sync::Arc;
/// Safe wrapper for the `ISteamUtils` interface.
#[derive(Debug, Clone)]
pub struct SteamUtils {
    inner: Arc<SteamUtilsInner>,
}

#[derive(Debug)]
struct SteamUtilsInner {
    ptr: *mut ISteamUtils,
}

impl SteamUtils {
    /// Creates a new `SteamUtils` instance from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be valid and remain valid for the lifetime of the `SteamUtils` instance.
    pub unsafe fn from_raw(ptr: *mut ISteamUtils) -> Self {
        Self {
            inner: Arc::new(SteamUtilsInner { ptr }),
        }
    }

    /// Returns the App ID for the current process.
    pub fn get_app_id(&self) -> Result<AppId_t, SteamClientError> {
        unsafe {
            let vtable = (*self.inner.ptr)
                .vtable
                .as_ref()
                .ok_or(SteamClientError::NullVtable)?;
            let app_id = (vtable.get_app_id)(self.inner.ptr);

            Ok(app_id)
        }
    }

    /// Checks if an API call has completed.
    pub fn is_api_call_completed(
        &self,
        api_call_handle: SteamAPICall_t,
    ) -> Result<bool, SteamClientError> {
        unsafe {
            let vtable = (*self.inner.ptr)
                .vtable
                .as_ref()
                .ok_or(SteamClientError::NullVtable)?;
            let mut b_failed = true;
            let completed =
                (vtable.is_api_call_completed)(self.inner.ptr, api_call_handle, &mut b_failed);

            if b_failed {
                dev_println!("is_api_call_completed failed");
                return Err(SteamClientError::UnknownError);
            }

            Ok(completed)
        }
    }

    /// Gets the result of an API call for a given callback ID.
    pub fn get_api_call_result<T>(
        &self,
        api_call_handle: SteamAPICall_t,
        callback_id: SteamCallbackId,
    ) -> Result<T, SteamClientError> {
        unsafe {
            let vtable = (*self.inner.ptr)
                .vtable
                .as_ref()
                .ok_or(SteamClientError::NullVtable)?;

            let mut b_failed = true;
            let mut result: T = std::mem::zeroed();
            let success = (vtable.get_api_call_result)(
                self.inner.ptr,
                api_call_handle,
                &mut result as *mut T as *mut c_void,
                size_of::<GlobalAchievementPercentagesReady_t>() as c_int,
                callback_id as c_int,
                &mut b_failed,
            );

            if b_failed {
                dev_println!("get_api_call_result failed");
                return Err(SteamClientError::UnknownError);
            }

            if !success {
                dev_println!("get_api_call_result not success");
                return Err(SteamClientError::UnknownError);
            }

            Ok(result)
        }
    }
}
