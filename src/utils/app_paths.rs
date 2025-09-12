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

use crate::utils::ipc_types::SamError;
use std::{env, path::PathBuf};

/// Returns the absolute path to the current executable, resolving symlinks.
pub fn get_executable_path() -> Result<PathBuf, SamError> {
    let exe = env::current_exe().map_err(|_| SamError::UnknownError)?;
    exe.canonicalize().map_err(|_| SamError::UnknownError)
}

/// Returns a valid directory for persistent app data (Linux).
#[inline]
#[cfg(target_os = "linux")]
pub fn get_app_cache_dir() -> Result<String, SamError> {
    use std::fs;
    if let Ok(snap_name) = env::var("SNAP_NAME") {
        if snap_name == "samrewritten" {
            let snap_user_common = env::var("SNAP_USER_COMMON").unwrap_or_else(|e| {
                log::warn!("SNAP_USER_COMMON not set: {e}, using /tmp");
                String::from("/tmp")
            });
            return Ok(snap_user_common);
        }
        // Most likely a dev config
        return Ok(".".to_owned());
    }
    // Non-snap release
    let home = env::var("HOME").unwrap_or_else(|e| {
        log::warn!("HOME not set: {e}, using /tmp");
        "/tmp".to_owned()
    });
    let folder = format!("{home}/.cache/samrewritten");
    if let Err(e) = fs::create_dir_all(&folder) {
        log::error!("Failed to create app cache dir {folder}: {e}");
        return Err(SamError::UnknownError);
    }
    Ok(folder)
}

/// Returns a valid directory for persistent app data (Windows).
#[inline]
#[cfg(target_os = "windows")]
pub fn get_app_cache_dir() -> Result<String, SamError> {
    let temp = env::temp_dir();
    temp.to_str().map(|s| s.to_owned()).ok_or(SamError::UnknownError)
}

/// Returns the path to the Steam client library (Linux).
#[inline]
#[cfg(target_os = "linux")]
pub fn get_steamclient_lib_path() -> Result<PathBuf, SamError> {
    use std::path::Path;
    if let Ok(real_home) = env::var("SNAP_REAL_HOME") {
        let path_str = format!("{real_home}/snap/steam/common/.local/share/Steam/linux64/steamclient.so");
        return Ok(Path::new(&path_str).to_owned());
    }
    let home = env::var("HOME").map_err(|_| SamError::UnknownError)?;
    let lib_paths = [
        format!("{home}/snap/steam/common/.local/share/Steam/linux64/steamclient.so"),
        format!("{home}/.steam/debian-installation/linux64/steamclient.so"),
        format!("{home}/.steam/sdk64/steamclient.so"),
        format!("{home}/.steam/steam/linux64/steamclient.so"),
        format!("{home}/.steam/root/linux64/steamclient.so"),
    ];
    for lib_path in &lib_paths {
        let path = Path::new(lib_path);
        if path.exists() {
            return Ok(path.into());
        }
    }
    Err(SamError::UnknownError)
}

/// Returns the path to the Steam client library (Windows).
#[inline]
#[cfg(target_os = "windows")]
pub fn get_steamclient_lib_path() -> Result<PathBuf, SamError> {
    use std::path::PathBuf;
    use winreg::RegKey;
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};

    const REG_PATH: &str = "SOFTWARE\\Valve\\Steam";
    const VALUE_NAME: &str = "SteamPath";

    // Try HKEY_CURRENT_USER first
    if let Ok(subkey) = RegKey::predef(HKEY_CURRENT_USER).open_subkey(REG_PATH) {
        if let Ok(value) = subkey.get_value::<String, _>(VALUE_NAME) {
            return Ok(PathBuf::from(value).join("steamclient64.dll"));
        }
    }

    // Fallback to HKEY_LOCAL_MACHINE
    if let Ok(subkey) = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(REG_PATH) {
        if let Ok(value) = subkey.get_value::<String, _>(VALUE_NAME) {
            return Ok(PathBuf::from(value).join("steamclient64.dll"));
        }
    }

    Err(SamError::UnknownError)
}

/// Returns the path to the user game stats schema file (Linux).
#[inline]
#[cfg(target_os = "linux")]
pub fn get_user_game_stats_schema_path(app_id: &u32) -> Result<String, SamError> {
    use std::path::Path;
    if let Ok(real_home) = env::var("SNAP_REAL_HOME") {
        return Ok(format!(
            "{real_home}/snap/steam/common/.local/share/Steam/appcache/stats/UserGameStatsSchema_{app_id}.bin"
        ));
    }
    let home = env::var("HOME").map_err(|_| SamError::UnknownError)?;
    let install_dirs = [
        format!("{home}/snap/steam/common/.local/share/Steam"),
        format!("{home}/.steam/debian-installation"),
        format!("{home}/.steam/steam"),
        format!("{home}/.steam/root"),
    ];
    for install_dir in &install_dirs {
        if Path::new(install_dir).exists() {
            return Ok(format!("{install_dir}/appcache/stats/UserGameStatsSchema_{app_id}.bin"));
        }
    }
    Err(SamError::UnknownError)
}

/// Returns the path to the user game stats schema file (Windows).
#[inline]
#[cfg(target_os = "windows")]
pub fn get_user_game_stats_schema_path(app_id: &u32) -> Result<String, SamError> {
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;

    let subkey = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey("SOFTWARE\\Valve\\Steam")
        .map_err(|_| SamError::UnknownError)?;

    let value = subkey
        .get_value::<String, &'static str>("SteamPath")
        .map_err(|_| SamError::UnknownError)?;

    Ok(format!("{value}/appcache/stats/UserGameStatsSchema_{app_id}.bin"))
}

/// Returns the path to the local app banner image (Linux).
#[inline]
#[cfg(target_os = "linux")]
pub fn get_local_app_banner_file_path(app_id: &u32) -> Result<String, SamError> {
    use std::path::Path;
    if let Ok(real_home) = env::var("SNAP_REAL_HOME") {
        return Ok(format!(
            "{real_home}/snap/steam/common/.local/share/Steam/appcache/librarycache/{app_id}/header.jpg"
        ));
    }
    let home = env::var("HOME").map_err(|_| SamError::UnknownError)?;
    let install_dirs = [
        format!("{home}/snap/steam/common/.local/share/Steam"),
        format!("{home}/.steam/debian-installation"),
        format!("{home}/.steam/steam"),
        format!("{home}/.steam/root"),
    ];
    for install_dir in &install_dirs {
        if Path::new(install_dir).exists() {
            return Ok(format!("{install_dir}/appcache/librarycache/{app_id}/header.jpg"));
        }
    }
    Err(SamError::UnknownError)
}

/// Returns the path to the local app banner image (Windows).
#[inline]
#[cfg(target_os = "windows")]
pub fn get_local_app_banner_file_path(app_id: &u32) -> Result<String, SamError> {
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;

    let subkey = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey("SOFTWARE\\Valve\\Steam")
        .map_err(|_| SamError::UnknownError)?;

    let value = subkey
        .get_value::<String, &'static str>("SteamPath")
        .map_err(|_| SamError::UnknownError)?;

    Ok(format!("{value}/appcache/librarycache/{app_id}/header.jpg"))
}
