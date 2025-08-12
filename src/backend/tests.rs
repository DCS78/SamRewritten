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
    fn get_achievements_with_callback() {
        let mut app_manager = match AppManager::new_connected(206690) {
            Ok(mgr) => mgr,
            Err(e) => {
                eprintln!("Failed to create app manager: {e}");
                return;
            }
        };
        let achievements = match app_manager.get_achievements() {
            Ok(ach) => ach,
            Err(e) => {
                eprintln!("Failed to get achievements: {e}");
                return;
            }
        };
        println!("{achievements:?}");
    }

    /// Test fetching stats for a known app.
    #[test]
    fn get_stats_no_message() {
        let mut app_manager = match AppManager::new_connected(480) {
            Ok(mgr) => mgr,
            Err(e) => {
                eprintln!("Failed to create app manager: {e}");
                return;
            }
        };
        let stats = match app_manager.get_statistics() {
            Ok(stats) => stats,
            Err(e) => {
                eprintln!("Failed to get stats: {e}");
                return;
            }
        };
        println!("{stats:?}");
    }

    /// Test resetting all stats for a known app.
    #[test]
    fn reset_stats_no_message() {
        let app_manager = match AppManager::new_connected(480) {
            Ok(mgr) => mgr,
            Err(e) => {
                eprintln!("Failed to create app manager: {e}");
                return;
            }
        };
        let success = match app_manager.reset_all_stats(true) {
            Ok(success) => success,
            Err(e) => {
                eprintln!("Failed to reset stats: {e}");
                return;
            }
        };
        println!("Success: {success:?}");
    }

    /// Test brute-forcing various app data keys for SteamApps001.
    #[test]
    fn brute_force_app001_keys() {
        // Find others on your own with the Steam command app_info_print
        let connected_steam = match ConnectedSteam::new() {
            Ok(cs) => cs,
            Err(e) => {
                eprintln!("Failed to create connected steam: {e}");
                return;
            }
        };
        let try_force = |key: &str| {
            let null_terminated_key = format!("{key}\0");
            let value = match connected_steam.apps_001.get_app_data(&220, &null_terminated_key) {
                Ok(val) => val,
                Err(e) => {
                    eprintln!("Failed to get app data for key: {key}: {e}");
                    "Failure".to_string()
                }
            };
            println!("{key}:\t {value}");
        };

        try_force(&SteamApps001AppDataKeys::Name.as_string());
        try_force(&SteamApps001AppDataKeys::Logo.as_string());
        try_force(&SteamApps001AppDataKeys::SmallCapsule("english").as_string());
        try_force("subscribed");

        try_force("metascore");
        try_force("metascore/score");
        try_force("metascorescore");
        try_force("metascorerating");
        try_force("metascore/rating");
        try_force("metascore_rating");
        try_force("metascore_rating");

        try_force("metacritic");
        try_force("metacritic/score");
        try_force("metacritic/url");
        try_force("metacriticurl/english");
        try_force("metacritic/url/english");
        try_force("metacriticscore");
        try_force("metacritic_score");
        try_force("metacriticrating");
        try_force("metacritic/rating");
        try_force("metacritic_rating");
        try_force("metacritic_rating");

        try_force("developer");
        try_force("developer/english");
        try_force("extended/developer");
        try_force("state");
        try_force("homepage");
        try_force("clienticon");
    }

    /// Test loading a binary KeyValue file from disk.
    #[test]
    fn keyval() {
        #[cfg(target_os = "linux")]
        let home = match env::var("HOME") {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Failed to get home directory: {e}");
                return;
            }
        };
        #[cfg(target_os = "linux")]
        let bin_file = PathBuf::from(
            home + "/snap/steam/common/.local/share/Steam/appcache/stats/UserGameStatsSchema_730.bin",
        );
        #[cfg(target_os = "windows")]
        let program_files = match env::var("ProgramFiles(x86)") {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Failed to get Program Files directory: {e}");
                return;
            }
        };
        #[cfg(target_os = "windows")]
        let bin_file =
            PathBuf::from(program_files + "\\Steam\\appcache\\stats\\UserGameStatsSchema_480.bin");

        let kv = match KeyValue::load_as_binary(bin_file) {
            Ok(kv) => kv,
            Err(e) => {
                eprintln!("Failed to load key value: {e}");
                return;
            }
        };
        println!("{kv:?}");
    }
}
