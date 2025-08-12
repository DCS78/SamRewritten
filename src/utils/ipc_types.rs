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

use interprocess::unnamed_pipe::Recver;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::io::Read;

/// Error types for orchestrator/app IPC.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum SamError {
    SerializationFailed,
    SteamConnectionFailed,
    AppListRetrievalFailed,
    SocketCommunicationFailed,
    AppMismatchError,
    UnknownError,
}

impl std::fmt::Display for SamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SamError::SerializationFailed => write!(f, "Sam error: Serialization failed"),
            SamError::SteamConnectionFailed => write!(f, "Sam error: Steam connection failed"),
            SamError::AppListRetrievalFailed => write!(f, "Sam error: App list retrieval failed"),
            SamError::UnknownError => write!(f, "Sam error: Unknown error"),
            SamError::SocketCommunicationFailed => {
                write!(f, "Sam error: SocketCommunication failed")
            }
            SamError::AppMismatchError => write!(f, "Sam error: App mismatch"),
        }
    }
}

impl std::error::Error for SamError {}

/// Commands sent from frontend to orchestrator/app.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum SteamCommand {
    GetOwnedAppList,
    LaunchApp(u32),
    StopApp(u32),
    StopApps,
    Shutdown,
    Status, // Ask for status of the process
    GetAchievements(u32),
    GetStats(u32),
    SetAchievement(u32, bool, String),
    SetIntStat(u32, String, i32),
    SetFloatStat(u32, String, f32),
    ResetStats(u32, bool),
}

/// Response from orchestrator/app to frontend.
#[derive(Serialize, Deserialize, Debug)]
pub enum SteamResponse<T> {
    Success(T),
    Error(SamError),
}

/// Trait for serializing/deserializing IPC messages with length prefix.
pub trait SamSerializable {
    /// Serializes the object to a length-prefixed JSON byte vector.
    fn sam_serialize(&self) -> Vec<u8>
    where
        Self: Sized + Serialize,
    {
        let serialized = match serde_json::to_string(&self) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[IPC] Serialization error: {e}");
                return vec![];
            }
        };
        let s_bytes = serialized.as_bytes();
        let length = s_bytes.len();
        let length_bytes = length.to_le_bytes();
        let mut result = Vec::with_capacity(length_bytes.len() + s_bytes.len());
        result.extend_from_slice(&length_bytes);
        result.extend_from_slice(s_bytes);
        result
    }

    /// Reads and deserializes a message from a Recver.
    fn from_recver(rx: &mut Recver) -> Result<Self, SamError>
    where
        Self: DeserializeOwned,
    {
        let mut buffer_len = [0u8; std::mem::size_of::<usize>()];
        match rx.read_exact(&mut buffer_len) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("[IPC] Error reading length from pipe: {e}");
                return Err(SamError::SocketCommunicationFailed);
            }
        }

        let data_length = usize::from_le_bytes(buffer_len);
        let mut buffer = vec![0u8; data_length];

        match rx.read_exact(&mut buffer) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("[IPC] Error reading message from pipe: {e}");
                return Err(SamError::SocketCommunicationFailed);
            }
        };

        let message = String::from_utf8_lossy(&buffer);
        let message: Self = match serde_json::from_str(&message) {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("[IPC] Failed to deserialize message: {e}");
                return Err(SamError::SerializationFailed);
            }
        };
        Ok(message)
    }
}

impl<T> SamSerializable for SteamResponse<T> where T: Sized + Serialize {}
impl SamSerializable for SteamCommand {}

impl<T> Into<Result<T, SamError>> for SteamResponse<T> {
    fn into(self) -> Result<T, SamError> {
        match self {
            SteamResponse::Success(data) => Ok(data),
            SteamResponse::Error(error) => Err(error),
        }
    }
}
