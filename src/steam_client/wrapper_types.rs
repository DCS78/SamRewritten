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

//! Contains error types and callback IDs for the Steam client wrappers.
/// Error type for Steam client wrapper operations.
#[derive(Debug, PartialEq)]
pub enum SteamClientError {
    /// The vtable pointer was null.
    NullVtable,
    /// Failed to create a Steam pipe.
    PipeCreationFailed,
    /// Failed to release a Steam pipe.
    PipeReleaseFailed,
    /// Failed to connect to the Steam server.
    UserConnectionFailed,
    /// Failed to create a Steam interface (with name).
    InterfaceCreationFailed(String),
    /// The requested app was not found.
    AppNotFound,
    /// An unknown error occurred.
    UnknownError,
}

impl std::fmt::Display for SteamClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SteamClientError::NullVtable => write!(f, "Steam client vtable is null"),
            SteamClientError::PipeCreationFailed => write!(f, "Failed to create steam pipe"),
            SteamClientError::PipeReleaseFailed => write!(f, "Failed to release steam pipe"),
            SteamClientError::UserConnectionFailed => {
                write!(f, "Failed to connect to steam server")
            }
            SteamClientError::InterfaceCreationFailed(name) => {
                write!(f, "Failed to create steam interface: {}", name)
            }
            SteamClientError::AppNotFound => write!(f, "App not found"),
            SteamClientError::UnknownError => write!(f, "Unknown Steam error"),
        }
    }
}

impl std::error::Error for SteamClientError {}

/// Enum of Steam callback IDs for wrapper event handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SteamCallbackId {
    /// Callback for user stats received.
    UserStatsReceived = 1101,
    /// Callback for global achievement percentages ready.
    GlobalAchievementPercentagesReady = 1110,
    /// Callback for global stats received.
    GlobalStatsReceived = 1112,
}
