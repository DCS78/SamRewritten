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
    let canonical = exe.canonicalize().map_err(|_| SamError::UnknownError)?;
    Ok(canonical)
}

/// Returns a valid directory for persistent app data (Linux).
#[inline]
#[cfg(target_os = "linux")]
pub fn get_app_cache_dir() -> Result<String, SamError> {
    use std::fs;
    if let Ok(snap_name) = env::var("SNAP_NAME") {
        if snap_name == "samrewritten" {
            let snap_user_common = match env::var("SNAP_USER_COMMON") {
                Ok(val) => val,
                Err(e) => {
                    log::warn!("SNAP_USER_COMMON not set: {e}, using /tmp");
                    String::from("/tmp")
                }
            };
            return Ok(snap_user_common);
        }
        // Most likely a dev config
        return Ok(".".to_owned());
    }
    // Non-snap release
    let home = match env::var("HOME") {
        Ok(val) => val,
        Err(e) => {
            log::warn!("HOME not set: {e}, using /tmp");
            "/tmp".to_owned()
        }
    };
    let folder = home + "/.cache/samrewritten";
    fs::create_dir_all(&folder).map_err(|e| {
        log::error!("Failed to create app cache dir {folder}: {e}");
        SamError::UnknownError
    })?;
    Ok(folder)
}

/// Returns a valid directory for persistent app data (Windows).
#[inline]
#[cfg(target_os = "windows")]
pub fn get_app_cache_dir() -> Result<String, SamError> {
    let temp = env::temp_dir();
    let temp_str = temp.to_str().ok_or(SamError::UnknownError)?;
    Ok(temp_str.to_owned())
}

/// Returns the path to the Steam client library (Linux).
#[inline]
#[cfg(target_os = "linux")]
pub fn get_steamclient_lib_path() -> Result<PathBuf, SamError> {
    use std::path::Path;
    if let Ok(real_home) = env::var("SNAP_REAL_HOME") {
        let path_str = real_home + "/snap/steam/common/.local/share/Steam/linux64/steamclient.so";
        return Ok(Path::new(&path_str).to_owned());
    }
    let home = env::var("HOME").map_err(|_| SamError::UnknownError)?;
    let lib_paths = [
        home.clone() + "/snap/steam/common/.local/share/Steam/linux64/steamclient.so",
        home.clone() + "/.steam/debian-installation/linux64/steamclient.so",
        home.clone() + "/.steam/sdk64/steamclient.so",
        home.clone() + "/.steam/steam/linux64/steamclient.so",
        home + "/.steam/root/linux64/steamclient.so",
    ];
    for lib_path in lib_paths {
        let path = Path::new(&lib_path);
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
            let path = PathBuf::from(value).join("steamclient64.dll");
            return Ok(path);
        }
    }

    // Fallback to HKEY_LOCAL_MACHINE
    if let Ok(subkey) = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(REG_PATH) {
        if let Ok(value) = subkey.get_value::<String, _>(VALUE_NAME) {
            let path = PathBuf::from(value).join("steamclient64.dll");
            return Ok(path);
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
        return Ok(real_home
            + "/snap/steam/common/.local/share/Steam/appcache/stats/UserGameStatsSchema_"
            + &app_id.to_string()
            + ".bin");
    }
    let home = env::var("HOME").map_err(|_| SamError::UnknownError)?;
    let install_dirs = [
        home.clone() + "/snap/steam/common/.local/share/Steam",
        home.clone() + "/.steam/debian-installation",
        home.clone() + "/.steam/steam",
        home + "/.steam/root",
    ];
    for install_dir in install_dirs {
        if Path::new(&install_dir).exists() {
            return Ok(install_dir + &format!("/appcache/stats/UserGameStatsSchema_{app_id}.bin"));
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

    Ok(value + &format!("/appcache/stats/UserGameStatsSchema_{app_id}.bin"))
}

/// Returns the path to the local app banner image (Linux).
#[inline]
#[cfg(target_os = "linux")]
pub fn get_local_app_banner_file_path(app_id: &u32) -> Result<String, SamError> {
    use std::path::Path;
    if let Ok(real_home) = env::var("SNAP_REAL_HOME") {
        return Ok(real_home
            + "/snap/steam/common/.local/share/Steam/appcache/librarycache/"
            + &app_id.to_string()
            + "/header.jpg");
    }
    let home = env::var("HOME").map_err(|_| SamError::UnknownError)?;
    let install_dirs = [
        home.clone() + "/snap/steam/common/.local/share/Steam",
        home.clone() + "/.steam/debian-installation",
        home.clone() + "/.steam/steam",
        home + "/.steam/root",
    ];
    for install_dir in install_dirs {
        if Path::new(&install_dir).exists() {
            return Ok(install_dir + &format!("/appcache/librarycache/{app_id}/header.jpg"));
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

    Ok(value + &format!("/appcache/librarycache/{app_id}/header.jpg"))
}
