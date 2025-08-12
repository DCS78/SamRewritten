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

#[cfg(debug_assertions)]
use crate::backend::stat_definitions::{AchievementInfo, StatInfo};
use crate::backend::{app_lister::AppLister, connected_steam::ConnectedSteam};
use crate::dev_println;
use crate::utils::{
    app_paths::get_executable_path,
    bidir_child::BidirChild,
    ipc_types::{SamError, SamSerializable, SteamCommand, SteamResponse},
};
use interprocess::unnamed_pipe::{Recver, Sender};
use std::{
    collections::HashMap,
    io::{Read, Write},
    process::Command,
};

/// Sends a command to a child app process and returns the response as bytes.
fn send_app_command(bidir: &mut BidirChild, command: SteamCommand) -> Vec<u8> {
    let command = command.sam_serialize();
    let mut buffer_len = [0u8; std::mem::size_of::<usize>()];
    if let Err(e) = bidir.tx.write_all(&command) {
        eprintln!("[ORCHESTRATOR] Error sending command: {e}");
        return SteamResponse::<()>::Error(SamError::SocketCommunicationFailed).sam_serialize();
    }

    if let Err(e) = bidir.rx.read_exact(&mut buffer_len) {
        eprintln!("[ORCHESTRATOR] Error reading length from pipe: {e}");
        return SteamResponse::<()>::Error(SamError::SocketCommunicationFailed).sam_serialize();
    }

    let data_length = usize::from_le_bytes(buffer_len);
    let mut buffer = vec![0u8; data_length];
    if let Err(e) = bidir.rx.read_exact(&mut buffer) {
        eprintln!("[ORCHESTRATOR] Error reading message from pipe: {e}");
        return SteamResponse::<()>::Error(SamError::SocketCommunicationFailed).sam_serialize();
    }

    let mut result = Vec::with_capacity(buffer_len.len() + buffer.len());
    result.extend_from_slice(&buffer_len);
    result.extend_from_slice(&buffer);
    result
}

/// Main backend event loop: handles Steam connection, app processes, and command dispatch.
pub fn orchestrator(parent_tx: &mut Sender, parent_rx: &mut Recver) -> i32 {
    let mut connected_steam: Option<ConnectedSteam> = None;
    let mut children_processes: HashMap<u32, BidirChild> = HashMap::new();

    loop {
        dev_println!("[ORCHESTRATOR] Main loop...");

        let message =
            SteamCommand::from_recver(parent_rx).expect("[ORCHESTRATOR] No message from pipe");

        dev_println!("[ORCHESTRATOR] Received message: {message:?}");

        if connected_steam.is_none() {
            if message == SteamCommand::Shutdown {
                let response = SteamResponse::Success(true).sam_serialize();
                parent_tx
                    .write_all(&response)
                    .expect("[ORCHESTRATOR] Failed to send response");
                dev_println!("[ORCHESTRATOR] Exiting");
                break 0;
            }

            connected_steam = match ConnectedSteam::new() {
                Ok(c) => Some(c),
                Err(e) => {
                    dev_println!("[ORCHESTRATOR] Error connecting to Steam: {e}");
                    let response: SteamResponse<String> =
                        SteamResponse::Error(SamError::SteamConnectionFailed);
                    let response = response.sam_serialize();
                    parent_tx
                        .write_all(&response)
                        .expect("[ORCHESTRATOR] Failed to send response");
                    continue;
                }
            };
        }

        let cs = connected_steam.as_mut().unwrap();
        let continue_running = process_command(message, parent_tx, &mut children_processes, cs);
        if !continue_running {
            break 0;
        }
    }
}

/// Handles a single SteamCommand, dispatching to the appropriate logic.
fn process_command(
    command: SteamCommand,
    tx: &mut Sender,
    children_processes: &mut HashMap<u32, BidirChild>,
    connected_steam: &mut ConnectedSteam,
) -> bool {
    match command {
        SteamCommand::GetOwnedAppList => {
            dev_println!("[ORCHESTRATOR] Received GetOwnedAppList");
            let apps_001 = &connected_steam.apps_001;
            let apps = &connected_steam.apps;
            let app_lister = AppLister::new(apps_001, apps);

            match app_lister.get_owned_apps() {
                Ok(apps) => {
                    let response = SteamResponse::Success(apps);
                    let response = response.sam_serialize();
                    tx.write_all(&response)
                        .expect("[ORCHESTRATOR] Failed to send response");
                }
                Err(e) => {
                    dev_println!("[ORCHESTRATOR] Error getting owned apps: {e}");
                    let response = SteamResponse::<()>::Error(e);
                    let response = response.sam_serialize();
                    tx.write_all(&response)
                        .expect("[ORCHESTRATOR] Failed to send response");
                }
            };
        }

        SteamCommand::LaunchApp(app_id) => {
            dev_println!("[ORCHESTRATOR] LaunchApp {}", app_id);

            #[cfg(debug_assertions)]
            if app_id == 0 {
                let response = SteamResponse::<bool>::Success(true).sam_serialize();
                tx.write_all(&response)
                    .expect("[APP SERVER] Failed to send response");
                return true;
            }

            // 1. Check if we own a process for this app
            if children_processes.contains_key(&app_id) {
                eprintln!("[ORCHESTRATOR] App {} is already running", app_id);
                let response: SteamResponse<()> = SteamResponse::Error(SamError::UnknownError);
                let response = response.sam_serialize();
                tx.write_all(&response)
                    .expect("[ORCHESTRATOR] Failed to send response");
                return true;
            }

            // 2. Launch the process
            let current_exe = get_executable_path();
            let child = BidirChild::new(Command::new(current_exe).arg(format!("--app={app_id}")))
                .expect("Could not create app server process");

            children_processes.insert(app_id, child);
            let response = SteamResponse::Success(true);
            let response = response.sam_serialize();
            tx.write_all(&response)
                .expect("[ORCHESTRATOR] Failed to send response");
        }

        SteamCommand::StopApp(app_id) => {
            #[cfg(debug_assertions)]
            if app_id == 0 {
                let response = SteamResponse::<bool>::Success(true).sam_serialize();
                tx.write_all(&response)
                    .expect("[APP SERVER] Failed to send response");
                return true;
            }

            if !children_processes.contains_key(&app_id) {
                eprintln!("[ORCHESTRATOR] App {} is not running", app_id);
                let response: SteamResponse<()> = SteamResponse::Error(SamError::UnknownError);
                let response = response.sam_serialize();
                tx.write_all(&response)
                    .expect("[ORCHESTRATOR] Failed to send response");
                return true;
            }

            let mut bidir_opt = children_processes.remove(&app_id);
            let bidir = bidir_opt.as_mut().unwrap();
            let response = send_app_command(bidir, SteamCommand::Shutdown);

            bidir
                .child
                .wait()
                .expect("[ORCHESTRATOR] Failed to wait child process]");

            tx.write_all(&response)
                .expect("[ORCHESTRATOR] Failed to send response");
        }

        SteamCommand::StopApps => {
            dev_println!("[ORCHESTRATOR] StopApps");

            for (app_id, child) in children_processes.iter_mut() {
                send_app_command(child, SteamCommand::Shutdown);
                dev_println!("[ORCHESTRATOR] Sent shutdown command to app {app_id}");
                child
                    .child
                    .wait()
                    .expect("[ORCHESTRATOR] Failed to wait child process]");
            }

            children_processes.clear();

            let response = SteamResponse::Success(true);
            let response = response.sam_serialize();
            tx.write_all(&response)
                .expect("[ORCHESTRATOR] Failed to send response");
        }

        SteamCommand::Shutdown => {
            for (app_id, child) in children_processes.iter_mut() {
                send_app_command(child, SteamCommand::Shutdown);
                dev_println!("[ORCHESTRATOR] Sent shutdown command to app {app_id}");
                child
                    .child
                    .wait()
                    .expect("[ORCHESTRATOR] Failed to wait child process]");
            }

            connected_steam.shutdown();

            let response = SteamResponse::Success(true);
            let response = response.sam_serialize();
            tx.write_all(&response)
                .expect("[ORCHESTRATOR] Failed to send response");
            return false;
        }

        SteamCommand::Status => {
            let response = SteamResponse::Success(true);
            let response = response.sam_serialize();
            tx.write_all(&response)
                .expect("[ORCHESTRATOR] Failed to send response");
        }

        SteamCommand::GetAchievements(app_id) => {
            #[cfg(debug_assertions)]
            if app_id == 0 {
                let mut ach_infos = vec![];
                for i in 1..1000 {
                    let ach_info = AchievementInfo {
                        id: format!("DEV_ACH_{i}"),
                        is_achieved: (i % 2) == 0,
                        name: format!("Development achievement {i}"),
                        global_achieved_percent: None,
                        permission: 0,
                        description: "Description".to_string(),
                        icon_locked: "".to_string(),
                        icon_normal: "".to_string(),
                        unlock_time: None,
                    };
                    ach_infos.push(ach_info);
                }

                let response =
                    SteamResponse::<Vec<AchievementInfo>>::Success(ach_infos).sam_serialize();
                tx.write_all(&response)
                    .expect("[APP SERVER] Failed to send response");
                return true;
            }

            if let Some(bidir) = children_processes.get_mut(&app_id) {
                let response = send_app_command(bidir, SteamCommand::GetAchievements(app_id));
                tx.write_all(&response)
                    .expect("[ORCHESTRATOR] Failed to send response");
            } else {
                let response = SteamResponse::<()>::Error(SamError::AppMismatchError);
                let response = response.sam_serialize();
                tx.write_all(&response)
                    .expect("[ORCHESTRATOR] Failed to send response");
            }
        }

        SteamCommand::GetStats(app_id) => {
            #[cfg(debug_assertions)]
            if app_id == 0 {
                let response = SteamResponse::<Vec<StatInfo>>::Success(vec![]).sam_serialize();
                tx.write_all(&response)
                    .expect("[APP SERVER] Failed to send response");
                return true;
            }

            if let Some(bidir) = children_processes.get_mut(&app_id) {
                let response = send_app_command(bidir, SteamCommand::GetStats(app_id));
                tx.write_all(&response)
                    .expect("[ORCHESTRATOR] Failed to send response");
            } else {
                let response = SteamResponse::<()>::Error(SamError::AppMismatchError);
                let response = response.sam_serialize();
                tx.write_all(&response)
                    .expect("[ORCHESTRATOR] Failed to send response");
            }
        }

        SteamCommand::SetAchievement(app_id, unlocked, achievement_id) => {
            #[cfg(debug_assertions)]
            if app_id == 0 {
                let response = SteamResponse::<bool>::Success(true).sam_serialize();
                tx.write_all(&response)
                    .expect("[APP SERVER] Failed to send response");
                return true;
            }

            if let Some(bidir) = children_processes.get_mut(&app_id) {
                let response = send_app_command(
                    bidir,
                    SteamCommand::SetAchievement(app_id, unlocked, achievement_id),
                );
                tx.write_all(&response)
                    .expect("[ORCHESTRATOR] Failed to send response");
            } else {
                let response = SteamResponse::<()>::Error(SamError::AppMismatchError);
                let response = response.sam_serialize();
                tx.write_all(&response)
                    .expect("[ORCHESTRATOR] Failed to send response");
            }
        }

        SteamCommand::SetIntStat(app_id, stat_id, value) => {
            if let Some(bidir) = children_processes.get_mut(&app_id) {
                let response =
                    send_app_command(bidir, SteamCommand::SetIntStat(app_id, stat_id, value));
                tx.write_all(&response)
                    .expect("[ORCHESTRATOR] Failed to send response");
            } else {
                let response = SteamResponse::<()>::Error(SamError::AppMismatchError);
                let response = response.sam_serialize();
                tx.write_all(&response)
                    .expect("[ORCHESTRATOR] Failed to send response");
            }
        }

        SteamCommand::SetFloatStat(app_id, stat_id, value) => {
            if let Some(bidir) = children_processes.get_mut(&app_id) {
                let response =
                    send_app_command(bidir, SteamCommand::SetFloatStat(app_id, stat_id, value));
                tx.write_all(&response)
                    .expect("[ORCHESTRATOR] Failed to send response");
            } else {
                let response = SteamResponse::<()>::Error(SamError::AppMismatchError);
                let response = response.sam_serialize();
                tx.write_all(&response)
                    .expect("[ORCHESTRATOR] Failed to send response");
            }
        }

        SteamCommand::ResetStats(app_id, achievements_too) => {
            if let Some(bidir) = children_processes.get_mut(&app_id) {
                let response =
                    send_app_command(bidir, SteamCommand::ResetStats(app_id, achievements_too));
                tx.write_all(&response)
                    .expect("[ORCHESTRATOR] Failed to send response");
            } else {
                let response = SteamResponse::<()>::Error(SamError::AppMismatchError);
                let response = response.sam_serialize();
                tx.write_all(&response)
                    .expect("[ORCHESTRATOR] Failed to send response");
            }
        }
    };

    true
}
