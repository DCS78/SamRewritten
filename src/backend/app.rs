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
    backend::{
        app_manager::AppManager,
        stat_definitions::{AchievementInfo, StatInfo},
    },
    dev_println,
    steam_client::steamworks_types::AppId_t,
    utils::ipc_types::{SamError, SamSerializable, SteamCommand, SteamResponse},
};
use interprocess::unnamed_pipe::{Recver, Sender};
use std::io::Write;
use serde::Serialize;

fn send_response<T: Serialize>(parent_tx: &mut Sender, response: SteamResponse<T>) {
    let response = response.sam_serialize();
    if let Err(_e) = parent_tx.write_all(&response) {
        dev_println!("[APP SERVER] Failed to send response: {e}");
    }
}

fn check_app_id(app_id_param: AppId_t, app_id: AppId_t, parent_tx: &mut Sender) -> bool {
    if app_id_param != app_id {
        dev_println!("[APP SERVER] App ID mismatch: {app_id_param} != {app_id}");
        send_response(parent_tx, SteamResponse::<()>::Error(SamError::AppMismatchError));
        return false;
    }
    true
}

/// Entrypoint for the app process. Handles IPC and delegates to AppManager.
pub fn app(app_id: AppId_t, parent_tx: &mut Sender, parent_rx: &mut Recver) -> i32 {
    let mut app_manager = AppManager::new_connected(app_id);

    #[cfg(debug_assertions)]
    if app_manager.as_ref().is_err() {
        dev_println!("[APP SERVER] Failed to connect to Steam");
    }

    loop {
        dev_println!("[APP SERVER] Main loop...");

        let command = match SteamCommand::from_recver(parent_rx) {
            Ok(cmd) => cmd,
            Err(_e) => {
                dev_println!("[APP SERVER] No message from pipe: {e}");
                break;
            }
        };

        if app_manager.as_ref().is_err() {
            send_response(parent_tx, SteamResponse::<()>::Error(SamError::SteamConnectionFailed));
            continue;
        }

        let app_manager = match app_manager.as_mut() {
            Ok(am) => am,
            Err(_e) => {
                dev_println!("[APP SERVER] app_manager is None: {e}");
                continue;
            }
        };

        match command {
            SteamCommand::Status => {
                send_response(parent_tx, SteamResponse::<bool>::Success(true));
            }

            SteamCommand::Shutdown => {
                app_manager.disconnect();
                send_response(parent_tx, SteamResponse::<bool>::Success(true));
                break;
            }

            SteamCommand::GetAchievements(app_id_param) => {
                if !check_app_id(app_id_param, app_id, parent_tx) { continue; }
                let response = match app_manager.get_achievements() {
                    Ok(achievements) => SteamResponse::Success(achievements),
                    Err(e) => SteamResponse::Error::<Vec<AchievementInfo>>(e),
                };
                send_response(parent_tx, response);
            }

            SteamCommand::GetStats(app_id_param) => {
                if !check_app_id(app_id_param, app_id, parent_tx) { continue; }
                let response = match app_manager.get_statistics() {
                    Ok(statistics) => SteamResponse::Success(statistics),
                    Err(e) => SteamResponse::Error::<Vec<StatInfo>>(e),
                };
                send_response(parent_tx, response);
            }

            SteamCommand::SetAchievement(app_id_param, unlocked, achievement_id) => {
                if !check_app_id(app_id_param, app_id, parent_tx) { continue; }
                let response = match app_manager.set_achievement(&achievement_id, unlocked) {
                    Ok(_) => SteamResponse::Success(true),
                    Err(e) => {
                        dev_println!("[APP SERVER] Error setting achievement: {e}");
                        SteamResponse::Error::<bool>(e)
                    }
                };
                send_response(parent_tx, response);
            }

            SteamCommand::SetIntStat(app_id_param, stat_id, value) => {
                if !check_app_id(app_id_param, app_id, parent_tx) { continue; }
                let response = match app_manager.set_stat_i32(&stat_id, value) {
                    Ok(result) => SteamResponse::Success(result),
                    Err(e) => {
                        dev_println!("[APP SERVER] Error setting int stat: {e}");
                        SteamResponse::Error::<bool>(e)
                    }
                };
                send_response(parent_tx, response);
            }

            SteamCommand::SetFloatStat(app_id_param, stat_id, value) => {
                if !check_app_id(app_id_param, app_id, parent_tx) { continue; }
                let response = match app_manager.set_stat_f32(&stat_id, value) {
                    Ok(result) => SteamResponse::Success(result),
                    Err(e) => {
                        dev_println!("[APP SERVER] Error setting float stat: {e}");
                        SteamResponse::Error::<bool>(e)
                    }
                };
                send_response(parent_tx, response);
            }

            SteamCommand::ResetStats(app_id_param, achievements_too) => {
                if !check_app_id(app_id_param, app_id, parent_tx) { continue; }
                let response = match app_manager.reset_all_stats(achievements_too) {
                    Ok(result) => SteamResponse::Success(result),
                    Err(e) => {
                        dev_println!("[APP SERVER] Error resetting stats: {e}");
                        SteamResponse::Error::<bool>(e)
                    }
                };
                send_response(parent_tx, response);
            }

            _ => {
                dev_println!("[APP SERVER] Received unknown command {command:?}");
                send_response(parent_tx, SteamResponse::<()>::Error(SamError::UnknownError));
            }
        }
    }

    dev_println!("[APP SERVER] Exiting");

    0
}
