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

//! Provides a safe Rust abstraction over the `ISteamUser` FFI interface.
use crate::steam_client::steam_user_vtable::ISteamUser;
use crate::steam_client::steamworks_types::CSteamID;
use crate::steam_client::wrapper_types::SteamClientError;
use std::sync::Arc;

/// Safe wrapper for the `ISteamUser` interface.
#[derive(Debug, Clone)]
pub struct SteamUser {
    inner: Arc<SteamUserInner>,
}

#[derive(Debug)]
struct SteamUserInner {
    ptr: *mut ISteamUser,
}

impl SteamUser {
    /// Creates a new `SteamUser` instance from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be valid and remain valid for the lifetime of the `SteamUser` instance.
    pub unsafe fn from_raw(ptr: *mut ISteamUser) -> Self {
        Self {
            inner: Arc::new(SteamUserInner { ptr }),
        }
    }

    /// Gets the SteamID for the current user (Unix).
    #[cfg(unix)]
    pub fn get_steam_id(&self) -> Result<CSteamID, SteamClientError> {
        unsafe {
            let vtable = (*self.inner.ptr)
                .vtable
                .as_ref()
                .ok_or(SteamClientError::NullVtable)?;

            let steam_id = (vtable.get_steam_id)(self.inner.ptr);
            Ok(steam_id)
        }
    }

    /// Gets the SteamID for the current user (Windows).
    #[cfg(windows)]
    pub fn get_steam_id(&self) -> Result<CSteamID, SteamClientError> {
        unsafe {
            let vtable = (*self.inner.ptr)
                .vtable
                .as_ref()
                .ok_or(SteamClientError::NullVtable)?;

            let mut id64 = 0u64;
            (vtable.get_steam_id)(self.inner.ptr, &mut id64);
            let steam_id = CSteamID { m_steamid: id64 };

            Ok(steam_id)
        }
    }
}
