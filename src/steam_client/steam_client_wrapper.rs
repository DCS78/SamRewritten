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

//! Provides a safe Rust abstraction over the `ISteamClient` FFI interface.
use crate::steam_client::steam_apps_001_vtable::{ISteamApps001, STEAMAPPS001_INTERFACE_VERSION};
use crate::steam_client::steam_apps_001_wrapper::SteamApps001;
use crate::steam_client::steam_apps_vtable::STEAMAPPS_INTERFACE_VERSION;
use crate::steam_client::steam_apps_wrapper::SteamApps;
use crate::steam_client::steam_client_vtable::ISteamClient;
use crate::steam_client::steam_user_stats_vtable::STEAMUSERSTATS_INTERFACE_VERSION;
use crate::steam_client::steam_user_stats_wrapper::SteamUserStats;
use crate::steam_client::steam_user_vtable::STEAMUSER_INTERFACE_VERSION;
use crate::steam_client::steam_user_wrapper::SteamUser;
use crate::steam_client::steam_utils_vtable::STEAMUTILS_INTERFACE_VERSION;
use crate::steam_client::steam_utils_wrapper::SteamUtils;
use crate::steam_client::steamworks_types::{
    HSteamPipe, HSteamUser, SteamFreeLastCallbackFn, SteamGetCallbackFn,
};
use crate::steam_client::wrapper_types::SteamClientError;
use libloading::Symbol;
use log;
use std::os::raw::c_char;
use std::sync::Arc;

/// Safe wrapper for the `ISteamClient` interface.
#[derive(Debug, Clone)]
pub struct SteamClient {
    inner: Arc<SteamClientInner>,
}

#[derive(Debug)]
struct SteamClientInner {
    ptr: *mut ISteamClient,
}

impl<'a> SteamClient {
    /// Creates a new `SteamClient` instance from a raw pointer and callback symbols.
    /// The pointer must be valid and remain valid for the lifetime of the `SteamClient` instance.
    pub unsafe fn from_raw(
        ptr: *mut ISteamClient,
        _callback_fn: Symbol<'a, SteamGetCallbackFn>,
        _free_callback_fn: Symbol<'a, SteamFreeLastCallbackFn>,
    ) -> Self {
        Self {
            inner: Arc::new(SteamClientInner { ptr }),
            // callback_fn,
            // free_callback_fn,
            // running_callback: false
        }
    }

    /// Creates a new Steam pipe and returns its handle.
    /// Returns `SteamClientError` if the vtable is null or pipe creation fails.
    pub fn create_steam_pipe(&self) -> Result<HSteamPipe, SteamClientError> {
        unsafe {
            let vtable = (*self.inner.ptr)
                .vtable
                .as_ref()
                .ok_or(SteamClientError::NullVtable)?;
            let pipe = (vtable.create_steam_pipe)(self.inner.ptr);
            if pipe == 0 {
                Err(SteamClientError::PipeCreationFailed)
            } else {
                Ok(pipe)
            }
        }
    }

    /// Releases a Steam pipe.
    /// Returns `SteamClientError` if the vtable is null or pipe release fails.
    pub fn release_steam_pipe(&self, pipe: HSteamPipe) -> Result<bool, SteamClientError> {
        unsafe {
            let vtable = (*self.inner.ptr)
                .vtable
                .as_ref()
                .ok_or(SteamClientError::NullVtable)?;
            let success = (vtable.release_steam_pipe)(self.inner.ptr, pipe);
            if success {
                Ok(success)
            } else {
                Err(SteamClientError::PipeReleaseFailed)
            }
        }
    }

    /// Releases a Steam user from a pipe.
    /// Panics if the vtable pointer is null.
    pub fn release_user(&self, pipe: HSteamPipe, user: HSteamUser) {
        unsafe {
            if let Some(vtable) = (*self.inner.ptr).vtable.as_ref() {
                (vtable.release_user)(self.inner.ptr, pipe, user);
            } else {
                log::error!("SteamClient vtable was null in release_user");
            }
        }
    }

    /// Connects to the global user for a given pipe.
    /// Returns `SteamClientError` if the vtable is null or user connection fails.
    pub fn connect_to_global_user(&self, pipe: HSteamPipe) -> Result<HSteamUser, SteamClientError> {
        unsafe {
            let vtable = (*self.inner.ptr)
                .vtable
                .as_ref()
                .ok_or(SteamClientError::NullVtable)?;
            let user = (vtable.connect_to_global_user)(self.inner.ptr, pipe);
            if user == 0 {
                Err(SteamClientError::UserConnectionFailed)
            } else {
                Ok(user)
            }
        }
    }

    /// Shuts down the client if all app pipes are closed.
    /// Returns `SteamClientError` if the vtable is null.
    pub fn shutdown_if_app_pipes_closed(&self) -> Result<bool, SteamClientError> {
        unsafe {
            let vtable = (*self.inner.ptr)
                .vtable
                .as_ref()
                .ok_or(SteamClientError::NullVtable)?;
            Ok((vtable.bshutdown_if_all_pipes_closed)(self.inner.ptr))
        }
    }

    /// Gets the `ISteamApps` interface for the given user and pipe.
    /// Returns `SteamClientError` if the vtable is null or interface creation fails.
    pub fn get_isteam_apps(
        &self,
        user: HSteamUser,
        pipe: HSteamPipe,
    ) -> Result<SteamApps, SteamClientError> {
        unsafe {
            let version = STEAMAPPS_INTERFACE_VERSION.as_ptr() as *const c_char;

            let vtable = (*self.inner.ptr)
                .vtable
                .as_ref()
                .ok_or(SteamClientError::NullVtable)?;
            let apps_ptr = (vtable.get_isteam_apps)(self.inner.ptr, user, pipe, version);

            if apps_ptr.is_null() {
                Err(SteamClientError::InterfaceCreationFailed(
                    "ISteamApps".to_owned(),
                ))
            } else {
                Ok(SteamApps::from_raw(apps_ptr))
            }
        }
    }

    /// Gets the `ISteamApps001` interface for the given user and pipe.
    /// Returns `SteamClientError` if the vtable is null or interface creation fails.
    pub fn get_isteam_apps_001(
        &self,
        user: HSteamUser,
        pipe: HSteamPipe,
    ) -> Result<SteamApps001, SteamClientError> {
        unsafe {
            let version = STEAMAPPS001_INTERFACE_VERSION.as_ptr() as *const c_char;

            let vtable = (*self.inner.ptr)
                .vtable
                .as_ref()
                .ok_or(SteamClientError::NullVtable)?;
            let apps_ptr = (vtable.get_isteam_apps)(self.inner.ptr, user, pipe, version);

            if apps_ptr.is_null() {
                Err(SteamClientError::InterfaceCreationFailed(
                    "ISteamApps001".to_owned(),
                ))
            } else {
                Ok(SteamApps001::from_raw(apps_ptr as *mut ISteamApps001))
            }
        }
    }

    /// Gets the `ISteamUtils` interface for the given pipe.
    /// Returns `SteamClientError` if the vtable is null or interface creation fails.
    pub fn get_isteam_utils(&self, pipe: HSteamPipe) -> Result<SteamUtils, SteamClientError> {
        unsafe {
            let version = STEAMUTILS_INTERFACE_VERSION.as_ptr() as *const c_char;

            let vtable = (*self.inner.ptr)
                .vtable
                .as_ref()
                .ok_or(SteamClientError::NullVtable)?;
            let utils_ptr = (vtable.get_isteam_utils)(self.inner.ptr, pipe, version);

            if utils_ptr.is_null() {
                Err(SteamClientError::InterfaceCreationFailed(
                    "ISteamUtils".to_owned(),
                ))
            } else {
                Ok(SteamUtils::from_raw(utils_ptr))
            }
        }
    }

    /// Gets the `ISteamUserStats` interface for the given user and pipe.
    /// Returns `SteamClientError` if the vtable is null or interface creation fails.
    pub fn get_isteam_user_stats(
        &self,
        user: HSteamUser,
        pipe: HSteamPipe,
    ) -> Result<SteamUserStats, SteamClientError> {
        unsafe {
            let version = STEAMUSERSTATS_INTERFACE_VERSION.as_ptr() as *const c_char;

            let vtable = (*self.inner.ptr)
                .vtable
                .as_ref()
                .ok_or(SteamClientError::NullVtable)?;
            let user_stats_ptr =
                (vtable.get_isteam_user_stats)(self.inner.ptr, user, pipe, version);

            if user_stats_ptr.is_null() {
                Err(SteamClientError::InterfaceCreationFailed(
                    "ISteamUtils".to_owned(),
                ))
            } else {
                Ok(SteamUserStats::from_raw(user_stats_ptr))
            }
        }
    }

    /// Gets the `ISteamUser` interface for the given user and pipe.
    /// Returns `SteamClientError` if the vtable is null or interface creation fails.
    pub fn get_isteam_user(
        &self,
        user: HSteamUser,
        pipe: HSteamPipe,
    ) -> Result<SteamUser, SteamClientError> {
        unsafe {
            let version = STEAMUSER_INTERFACE_VERSION.as_ptr() as *const c_char;

            let vtable = (*self.inner.ptr)
                .vtable
                .as_ref()
                .ok_or(SteamClientError::NullVtable)?;
            let user_ptr = (vtable.get_isteam_user)(self.inner.ptr, user, pipe, version);

            if user_ptr.is_null() {
                Err(SteamClientError::InterfaceCreationFailed(
                    "ISteamUser".to_owned(),
                ))
            } else {
                Ok(SteamUser::from_raw(user_ptr))
            }
        }
    }
}
