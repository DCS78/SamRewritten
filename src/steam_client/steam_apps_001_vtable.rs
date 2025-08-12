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

use crate::steam_client::steamworks_types::AppId_t;
use std::os::raw::{c_char, c_int};

/// VTable for the ISteamApps001 interface (Steam API).
#[repr(C)]
pub struct ISteamApps001VTable {
    /// Gets app data for a given AppId and key.
    pub get_app_data: unsafe extern "C" fn(
        *mut ISteamApps001,
        AppId_t,
        *const c_char,
        *mut c_char,
        c_int,
    ) -> c_int,
}

/// Opaque ISteamApps001 interface pointer.
#[repr(C)]
pub struct ISteamApps001 {
    pub vtable: *const ISteamApps001VTable,
}

/// The interface version string for ISteamApps001.
pub const STEAMAPPS001_INTERFACE_VERSION: &str = "STEAMAPPS_INTERFACE_VERSION001\0";
