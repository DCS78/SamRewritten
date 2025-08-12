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

use crate::backend::app_lister::AppModel;
use crate::backend::stat_definitions::{AchievementInfo, StatInfo};
use crate::dev_println;
use crate::frontend::DEFAULT_PROCESS;
use crate::utils::ipc_types::{SamError, SamSerializable, SteamCommand, SteamResponse};
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::io::{Read, Write};

/// Trait for sending a request to the orchestrator and receiving a typed response.
pub trait Request: Into<SteamCommand> + Debug + Clone {
    type Response: DeserializeOwned;

    fn request(self) -> Result<Self::Response, SamError> {
        let mut guard = match DEFAULT_PROCESS.write() {
            Ok(g) => g,
            Err(e) => {
                eprintln!("[CLIENT] Failed to lock DEFAULT_PROCESS: {e}");
                return Err(SamError::SocketCommunicationFailed);
            }
        };
        if let Some(ref mut bidir) = *guard {
            let command: SteamCommand = self.clone().into();
            dev_println!("[CLIENT] Sending command: {:?}", command);
            let command = command.sam_serialize();
            if let Err(e) = bidir.tx.write_all(&command) {
                eprintln!("[CLIENT] Error writing command to pipe: {e}");
                return Err(SamError::SocketCommunicationFailed);
            }

            let mut buffer_len = [0u8; std::mem::size_of::<usize>()];
            match bidir.rx.read_exact(&mut buffer_len) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("[CLIENT] Error reading length from pipe: {e}");
                    return Err(SamError::SocketCommunicationFailed);
                }
            }

            let data_length = usize::from_le_bytes(buffer_len);
            let mut buffer = vec![0u8; data_length];
            match bidir.rx.read_exact(&mut buffer) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("[CLIENT] Error reading message from pipe: {e}");
                    return Err(SamError::SocketCommunicationFailed);
                }
            };

            let message = String::from_utf8_lossy(&buffer);
            serde_json::from_str::<SteamResponse<Self::Response>>(&message)
                .map_err(|error| {
                    eprintln!("[CLIENT] Response deserialization failed: {error}");
                    SamError::SocketCommunicationFailed
                })
                .and_then(|response| response.into())
        } else {
            eprintln!("[CLIENT] No orchestrator process to shutdown");
            Err(SamError::SocketCommunicationFailed)
        }
    }
}

/// Request to get the list of owned apps.
#[derive(Debug, Clone)]
pub struct GetOwnedAppList;

/// Request to shut down the orchestrator.
#[derive(Debug, Clone)]
pub struct Shutdown;

/// Request to launch an app by app_id.
#[derive(Debug, Clone)]
pub struct LaunchApp {
    pub app_id: u32,
}

/// Request to stop a specific app by app_id.
#[derive(Debug, Clone)]
pub struct StopApp {
    pub app_id: u32,
}

/// Request to stop all running apps.
#[derive(Debug, Clone)]
 #[allow(dead_code)]
 pub struct StopApps;

/// Request to get achievements for an app.
#[derive(Debug, Clone)]
pub struct GetAchievements {
    pub app_id: u32,
}

/// Request to get stats for an app.
#[derive(Debug, Clone)]
pub struct GetStats {
    pub app_id: u32,
}

/// Request to set an achievement's unlocked state.
#[derive(Debug, Clone)]
pub struct SetAchievement {
    pub app_id: u32,
    pub achievement_id: String,
    pub unlocked: bool,
}

/// Request to set an integer stat value.
#[derive(Debug, Clone)]
pub struct SetIntStat {
    pub app_id: u32,
    pub stat_id: String,
    pub value: i32,
}

/// Request to set a float stat value.
#[derive(Debug, Clone)]
pub struct SetFloatStat {
    pub app_id: u32,
    pub stat_id: String,
    pub value: f32,
}

/// Request to reset stats (and optionally achievements) for an app.
#[derive(Debug, Clone)]
pub struct ResetStats {
    pub app_id: u32,
    pub achievements_too: bool,
}

impl Request for GetOwnedAppList {
    type Response = Vec<AppModel>;
}

impl Request for Shutdown {
    type Response = bool;
}

impl Request for LaunchApp {
    type Response = bool;
}

impl Request for StopApp {
    type Response = bool;
}

impl Request for StopApps {
    type Response = bool;
}

impl Request for GetAchievements {
    type Response = Vec<AchievementInfo>;
}

impl Request for GetStats {
    type Response = Vec<StatInfo>;
}

impl Request for SetAchievement {
    type Response = bool;
}

impl Request for SetIntStat {
    type Response = bool;
}

impl Request for SetFloatStat {
    type Response = bool;
}

impl Request for ResetStats {
    type Response = bool;
}

impl Into<SteamCommand> for GetOwnedAppList {
    fn into(self) -> SteamCommand {
        SteamCommand::GetOwnedAppList
    }
}

impl Into<SteamCommand> for Shutdown {
    fn into(self) -> SteamCommand {
        SteamCommand::Shutdown
    }
}

impl Into<SteamCommand> for LaunchApp {
    fn into(self) -> SteamCommand {
        SteamCommand::LaunchApp(self.app_id)
    }
}

impl Into<SteamCommand> for StopApp {
    fn into(self) -> SteamCommand {
        SteamCommand::StopApp(self.app_id)
    }
}

impl Into<SteamCommand> for StopApps {
    fn into(self) -> SteamCommand {
        SteamCommand::StopApps
    }
}

impl Into<SteamCommand> for GetAchievements {
    fn into(self) -> SteamCommand {
        SteamCommand::GetAchievements(self.app_id)
    }
}

impl Into<SteamCommand> for GetStats {
    fn into(self) -> SteamCommand {
        SteamCommand::GetStats(self.app_id)
    }
}

impl Into<SteamCommand> for SetAchievement {
    fn into(self) -> SteamCommand {
        SteamCommand::SetAchievement(self.app_id, self.unlocked, self.achievement_id)
    }
}

impl Into<SteamCommand> for SetIntStat {
    fn into(self) -> SteamCommand {
        SteamCommand::SetIntStat(self.app_id, self.stat_id, self.value)
    }
}

impl Into<SteamCommand> for SetFloatStat {
    fn into(self) -> SteamCommand {
        SteamCommand::SetFloatStat(self.app_id, self.stat_id, self.value)
    }
}

impl Into<SteamCommand> for ResetStats {
    fn into(self) -> SteamCommand {
        SteamCommand::ResetStats(self.app_id, self.achievements_too)
    }
}
