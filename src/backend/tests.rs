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

#[cfg(test)]
mod tests {
    use crate::backend::{
        app_manager::AppManager, connected_steam::ConnectedSteam, key_value::KeyValue,
    };
    use crate::steam_client::steam_apps_001_wrapper::SteamApps001AppDataKeys;
    use std::{env, path::PathBuf};

    /// Test fetching achievements for a known app.
    #[test]
    fn get_achievements_with_callback() -> Result<(), String> {
        let mut app_manager = AppManager::new_connected(206690)
            .map_err(|e| format!("Failed to create app manager: {e}"))?;
        let achievements = app_manager.get_achievements()
            .map_err(|e| format!("Failed to get achievements: {e}"))?;
        // You may want to assert something more specific if you know the expected result
        assert!(!achievements.is_empty(), "Achievements should not be empty");
        Ok(())
    }

    /// Test fetching stats for a known app.
    #[test]
    fn get_stats_no_message() -> Result<(), String> {
        let mut app_manager = AppManager::new_connected(480)
            .map_err(|e| format!("Failed to create app manager: {e}"))?;
        let stats = app_manager.get_statistics()
            .map_err(|e| format!("Failed to get stats: {e}"))?;
        // You may want to assert something more specific if you know the expected result
        assert!(!stats.is_empty(), "Stats should not be empty");
        Ok(())
    }

    /// Test resetting all stats for a known app.
    #[test]
    fn reset_stats_no_message() -> Result<(), String> {
        let app_manager = AppManager::new_connected(480)
            .map_err(|e| format!("Failed to create app manager: {e}"))?;
        let success = app_manager.reset_all_stats(true)
            .map_err(|e| format!("Failed to reset stats: {e}"))?;
        assert!(success, "Reset stats should return true");
        Ok(())
    }

    /// Test brute-forcing various app data keys for SteamApps001.
    #[test]
    fn brute_force_app001_keys() -> Result<(), String> {
        let connected_steam = ConnectedSteam::new()
            .map_err(|e| format!("Failed to create connected steam: {e}"))?;
        let try_force = |key: &str| {
            let null_terminated_key = format!("{key}\0");
            let value = connected_steam.apps_001.get_app_data(&220, &null_terminated_key);
            // Not asserting here, just checking that it doesn't panic
            value.is_ok()
        };
        assert!(try_force(&SteamApps001AppDataKeys::Name.as_string()));
        assert!(try_force(&SteamApps001AppDataKeys::Logo.as_string()));
        assert!(try_force(&SteamApps001AppDataKeys::SmallCapsule("english").as_string()));
        assert!(try_force("subscribed"));
        Ok(())
    }

    /// Test loading a binary KeyValue file from disk.
    #[test]
    fn keyval() -> Result<(), String> {
        #[cfg(target_os = "linux")]
        let home = env::var("HOME").map_err(|e| format!("Failed to get home directory: {e}"))?;
        #[cfg(target_os = "linux")]
        let bin_file = PathBuf::from(
            home + "/snap/steam/common/.local/share/Steam/appcache/stats/UserGameStatsSchema_730.bin",
        );
        #[cfg(target_os = "windows")]
        let program_files = env::var("ProgramFiles(x86)").map_err(|e| format!("Failed to get Program Files directory: {e}"))?;
        #[cfg(target_os = "windows")]
        let bin_file =
            PathBuf::from(program_files + "\\Steam\\appcache\\stats\\UserGameStatsSchema_480.bin");

        let kv = KeyValue::load_as_binary(bin_file)
            .map_err(|e| format!("Failed to load key value: {e}"))?;
        // You may want to assert something more specific if you know the expected result
        assert!(!format!("{kv:?}").is_empty(), "KeyValue should not be empty");
        Ok(())
    }
}
