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

//! Provides raw FFI bindings for the `ISteamApps` interface vtable.
//! All functions in this vtable are unsafe and must be called with valid pointers and correct types as expected by the Steamworks API.
use crate::steam_client::steamworks_types::{AppId_t, CSteamID, DepotId_t, SteamAPICall_t};
use std::os::raw::{c_char, c_int};

/// Raw vtable for the ISteamApps interface.
/// All function pointers must be called with valid pointers and correct types as expected by the Steamworks API.
#[repr(C)]
pub struct ISteamAppsVTable {
    /// Returns true if the user is subscribed to the current app.
    pub b_is_subscribed: unsafe extern "C" fn(*mut ISteamApps) -> bool,
    /// Returns true if the current app is in low violence mode.
    pub b_is_low_violence: unsafe extern "C" fn(*mut ISteamApps) -> bool,
    /// Returns true if running in a cybercafe environment.
    pub b_is_cybercafe: unsafe extern "C" fn(*mut ISteamApps) -> bool,
    /// Returns true if the user is VAC banned.
    pub b_is_vac_banned: unsafe extern "C" fn(*mut ISteamApps) -> bool,
    /// Returns the current game language as a C string.
    pub get_current_game_language: unsafe extern "C" fn(*mut ISteamApps) -> *const c_char,
    /// Returns available game languages as a C string.
    pub get_available_game_languages: unsafe extern "C" fn(*mut ISteamApps) -> *const c_char,
    /// Returns true if the user is subscribed to the given app.
    pub b_is_subscribed_app: unsafe extern "C" fn(*mut ISteamApps, AppId_t) -> bool,
    /// Returns true if the given DLC is installed.
    pub b_is_dlc_installed: unsafe extern "C" fn(*mut ISteamApps, AppId_t) -> bool,
    /// Returns the earliest purchase Unix time for the given app.
    pub get_earliest_purchase_unix_time: unsafe extern "C" fn(*mut ISteamApps, AppId_t) -> u32,
    /// Returns true if the user is subscribed from a free weekend.
    pub b_is_subscribed_from_free_weekend: unsafe extern "C" fn(*mut ISteamApps) -> bool,
    /// Returns the number of DLCs for the current app.
    pub get_dlc_count: unsafe extern "C" fn(*mut ISteamApps) -> c_int,
    /// Retrieves DLC data by index.
    pub b_get_dlc_data_by_index: unsafe extern "C" fn(
        *mut ISteamApps,
        c_int,
        *mut AppId_t,
        *mut bool,
        *mut c_char,
        c_int,
    ) -> bool,
    /// Installs the given DLC.
    pub install_dlc: unsafe extern "C" fn(*mut ISteamApps, AppId_t),
    /// Uninstalls the given DLC.
    pub uninstall_dlc: unsafe extern "C" fn(*mut ISteamApps, AppId_t),
    /// Requests a proof of purchase key for the given app.
    pub request_app_proof_of_purchase_key: unsafe extern "C" fn(*mut ISteamApps, AppId_t),
    /// Gets the current beta name into the provided buffer.
    pub get_current_beta_name: unsafe extern "C" fn(*mut ISteamApps, *mut c_char, c_int) -> bool,
    /// Marks content as corrupt.
    pub mark_content_corrupt: unsafe extern "C" fn(*mut ISteamApps, bool) -> bool,
    /// Gets installed depots for the given app.
    pub get_installed_depots:
        unsafe extern "C" fn(*mut ISteamApps, AppId_t, *mut DepotId_t, u32) -> u32,
    /// Gets the install directory for the given app.
    pub get_app_install_dir:
        unsafe extern "C" fn(*mut ISteamApps, AppId_t, *mut c_char, u32) -> u32,
    /// Returns true if the given app is installed.
    pub b_is_app_installed: unsafe extern "C" fn(*mut ISteamApps, AppId_t) -> bool,
    /// Gets the SteamID of the app owner.
    pub get_app_owner: unsafe extern "C" fn(*mut ISteamApps) -> CSteamID,
    /// Gets a launch query parameter by key.
    pub get_launch_query_param:
        unsafe extern "C" fn(*mut ISteamApps, *const c_char) -> *const c_char,
    /// Gets DLC download progress for the given app.
    pub get_dlc_download_progress:
        unsafe extern "C" fn(*mut ISteamApps, AppId_t, *mut u64, *mut u64) -> bool,
    /// Gets the build ID of the app.
    pub get_app_build_id: unsafe extern "C" fn(*mut ISteamApps) -> c_int,
    /// Requests all proof of purchase keys.
    pub request_all_proof_of_purchase_keys: unsafe extern "C" fn(*mut ISteamApps),
    /// Gets file details for the given file name.
    pub get_file_details: unsafe extern "C" fn(*mut ISteamApps, *const c_char) -> SteamAPICall_t,
    /// Gets the launch command line into the provided buffer.
    pub get_launch_command_line: unsafe extern "C" fn(*mut ISteamApps, *mut c_char, c_int) -> c_int,
    /// Returns true if the user is subscribed from family sharing.
    pub b_is_subscribed_from_family_sharing: unsafe extern "C" fn(*mut ISteamApps) -> bool,
    /// Returns true if the app is a timed trial, and outputs start/end times.
    pub b_is_timed_trial: unsafe extern "C" fn(*mut ISteamApps, *mut u32, *mut u32) -> bool,
    /// Sets the DLC context for the given app.
    pub set_dlc_context: unsafe extern "C" fn(*mut ISteamApps, AppId_t) -> bool,
}

/// Opaque struct representing an ISteamApps instance.
#[repr(C)]
pub struct ISteamApps {
    /// Pointer to the vtable for this instance.
    pub vtable: *const ISteamAppsVTable,
}

/// The interface version string for ISteamApps.
pub const STEAMAPPS_INTERFACE_VERSION: &str = "STEAMAPPS_INTERFACE_VERSION008\0";
