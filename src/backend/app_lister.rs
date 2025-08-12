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

use crate::{
    dev_println,
    steam_client::{
        steam_apps_001_wrapper::{SteamApps001, SteamApps001AppDataKeys},
        steam_apps_wrapper::SteamApps,
        steamworks_types::AppId_t,
    },
    utils::{app_paths::get_app_cache_dir, ipc_types::SamError},
};
use log;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    fs::{self, File},
    io::BufReader,
    str::FromStr,
    time::{Duration, SystemTime},
};

/// Loads, parses, and manages the list of Steam apps for the user.
#[derive(Debug)]
pub struct AppLister<'a> {
    app_list_url: String,
    app_list_local: String,
    current_language: String,
    steam_apps_001: &'a SteamApps001,
    steam_apps: &'a SteamApps,
}

/// Model for a Steam app.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppModel {
    pub app_id: AppId_t,
    pub app_name: String,
    pub image_url: Option<String>,
    pub app_type: AppModelType,
    pub developer: String,
    pub metacritic_score: Option<u8>,
}

/// Enum for Steam app type.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppModelType {
    App,
    Mod,
    Demo,
    Junk,
}

impl Display for AppModelType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::App => write!(f, "App"),
            Self::Mod => write!(f, "Mod"),
            Self::Demo => write!(f, "Demo"),
            Self::Junk => write!(f, "Junk"),
        }
    }
}

impl FromStr for AppModelType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "app" => Ok(Self::App),
            "mod" => Ok(Self::Mod),
            "demo" => Ok(Self::Demo),
            "junk" => Ok(Self::Junk),
            _ => Err(format!("'{}' is not a valid AppModelType", s)),
        }
    }
}

/// XML representation of a game entry.
#[derive(Deserialize, Debug, Clone)]
pub struct XmlGame {
    #[serde(rename = "$text")]
    pub app_id: u32,
    #[serde(rename = "@type")]
    pub app_type: Option<String>,
}

/// XML representation of a list of games.
#[derive(Deserialize, Debug, Clone)]
struct XmlGames {
    #[serde(rename = "game")]
    pub games: Vec<XmlGame>,
}
impl<'a> AppLister<'a> {
    /// Create a new AppLister.
    pub fn new(steam_apps_001: &'a SteamApps001, steam_apps: &'a SteamApps) -> Self {
        let cache_dir = match get_app_cache_dir() {
            Ok(dir) => dir,
            Err(e) => {
                log::error!("Failed to get app cache dir: {e}");
                String::from("/tmp")
            }
        };
        let app_list_url = match std::env::var("APP_LIST_URL") {
            Ok(val) => val,
            Err(_) => {
                log::warn!("APP_LIST_URL not set, using default");
                "https://gib.me/sam/games.xml".to_string()
            }
        };
        let app_list_local = match std::env::var("APP_LIST_LOCAL") {
            Ok(val) => format!("{}{}", cache_dir, val),
            Err(e) => {
                log::warn!("Failed to get APP_LIST_LOCAL: {e}, using default");
                format!("{}{}", cache_dir, "/apps.xml")
            }
        };
        let current_language = steam_apps.get_current_game_language();

        Self {
            app_list_url,
            app_list_local,
            current_language,
            steam_apps_001,
            steam_apps,
        }
    }

    /// Download the app list as a string from the remote URL.
    fn download_app_list_str(&self) -> Result<String, SamError> {
        dev_println!(
            "[ORCHESTRATOR] Downloading app list from:  {}",
            &self.app_list_url
        );
        let response = reqwest::blocking::get(&self.app_list_url)
            .map_err(|_| SamError::AppListRetrievalFailed)?
            .text()
            .map_err(|_| SamError::AppListRetrievalFailed)?;
        Ok(response)
    }

    /// Load the app list from the local XML file.
    fn load_app_list_file(&self) -> Result<XmlGames, SamError> {
        let file =
            File::open(&self.app_list_local).map_err(|_| SamError::AppListRetrievalFailed)?;
        let reader = BufReader::new(file);
        let xml_data: XmlGames =
            quick_xml::de::from_reader(reader).map_err(|_| SamError::AppListRetrievalFailed)?;
        Ok(xml_data)
    }

    /// Load the app list from a string.
    fn load_app_list_str(&self, source: &str) -> Result<XmlGames, SamError> {
        let xml_data: XmlGames =
            quick_xml::de::from_str(source).map_err(|_| SamError::AppListRetrievalFailed)?;
        Ok(xml_data)
    }

    /// Get the XML games, updating from remote if needed.
    fn get_xml_games(&self) -> Result<XmlGames, SamError> {
        const ONE_WEEK_SECS: u64 = 7 * 24 * 60 * 60;
        let should_update = match fs::metadata(&self.app_list_local) {
            Ok(metadata) => {
                let last_update = metadata
                    .modified()
                    .map_err(|_| SamError::AppListRetrievalFailed)?;
                let one_week_ago = SystemTime::now() - Duration::from_secs(ONE_WEEK_SECS);
                last_update < one_week_ago
            }
            Err(_) => true,
        };

        let xml_games = if should_update {
            let app_list_str = self.download_app_list_str()?;
            let xml_games = self.load_app_list_str(&app_list_str)?;
            dev_println!(
                "[ORCHESTRATOR] App list loaded. Saving in:  {}",
                &self.app_list_local
            );
            fs::write(&self.app_list_local, &app_list_str)
                .map_err(|_| SamError::AppListRetrievalFailed)?;
            xml_games
        } else {
            dev_println!("[ORCHESTRATOR] Loading app list from local location");
            self.load_app_list_file()?
        };

        Ok(xml_games)
    }

    /// Get the image URL for a given app.
    fn get_app_image_url(&self, app_id: &AppId_t) -> Option<String> {
        let try_capsule = |lang| {
            match self.steam_apps_001.get_app_data(
                app_id,
                &SteamApps001AppDataKeys::SmallCapsule(lang).as_string(),
            ) {
                Ok(val) => val,
                Err(e) => {
                    log::warn!("Failed to get SmallCapsule for app {}: {e}", app_id);
                    String::new()
                }
            }
        };
        let candidate = try_capsule(&self.current_language);
        if !candidate.is_empty() {
            return Some(format!(
                "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/{app_id}/{candidate}"
            ));
        }
        if self.current_language != "english" {
            let candidate = try_capsule("english");
            if !candidate.is_empty() {
                return Some(format!(
                    "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/{app_id}/{candidate}"
                ));
            }
        }
        let candidate = match self
            .steam_apps_001
            .get_app_data(app_id, &SteamApps001AppDataKeys::Logo.as_string()) {
            Ok(val) => val,
            Err(e) => {
                log::warn!("Failed to get Logo for app {}: {e}", app_id);
                String::new()
            }
        };
        if !candidate.is_empty() {
            return Some(format!(
                "https://cdn.steamstatic.com/steamcommunity/public/images/apps/{app_id}/{candidate}.jpg"
            ));
        }
        dev_println!("[ORCHESTRATOR] Failed to find image for app {}", app_id);
        None
    }

    /// Get an AppModel for a given app_id and XmlGame.
    pub fn get_app(&self, app_id: AppId_t, xml_game: &XmlGame) -> Result<AppModel, SamError> {
        let app_name = self
            .steam_apps_001
            .get_app_data(&app_id, &SteamApps001AppDataKeys::Name.as_string())
            .map_err(|e| {
                log::error!("Failed to get app name for {}: {e}", app_id);
                SamError::AppListRetrievalFailed
            })?;
        let developer = match self
            .steam_apps_001
            .get_app_data(&app_id, &SteamApps001AppDataKeys::Developer.as_string()) {
            Ok(val) => val,
            Err(e) => {
                log::warn!("Failed to get developer for {}: {e}", app_id);
                "Unknown".to_string()
            }
        };
        let metacritic_score: Option<u8> = self
            .steam_apps_001
            .get_app_data(
                &app_id,
                &SteamApps001AppDataKeys::MetacriticScore.as_string(),
            )
            .ok()
            .and_then(|s| s.parse().ok());
        let image_url = self.get_app_image_url(&app_id);
        let app_type = xml_game
            .app_type
            .as_deref()
            .map_or(Ok(AppModelType::App), AppModelType::from_str)
            .map_err(|_| SamError::AppListRetrievalFailed)?;
        Ok(AppModel {
            app_id,
            app_name,
            image_url,
            app_type,
            developer,
            metacritic_score,
        })
    }

    /// Get all owned apps as AppModel.
    pub fn get_owned_apps(&self) -> Result<Vec<AppModel>, SamError> {
        let xml_games = self.get_xml_games()?;
        let mut models = Vec::new();
        for xml_game in xml_games.games {
            let app_id: AppId_t = xml_game.app_id;
            match self.steam_apps.is_subscribed_app(app_id) {
                Ok(true) => {
                    let app = self.get_app(app_id, &xml_game)?;
                    models.push(app);
                }
                Ok(false) => continue,
                Err(e) => {
                    log::warn!("Failed to check is_subscribed_app for {}: {e}", app_id);
                    continue;
                }
            }
        }
        Ok(models)
    }
}
