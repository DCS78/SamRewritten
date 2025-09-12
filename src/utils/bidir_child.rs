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
#[cfg(unix)]
use interprocess::unnamed_pipe::pipe;
use interprocess::unnamed_pipe::{Recver, Sender};
#[cfg(unix)]
use std::os::fd::IntoRawFd;
#[cfg(windows)]
use std::os::windows::io::{AsRawHandle, OwnedHandle};
use std::process::{Child, Command};

/// Represents a child process with bidirectional unnamed pipes for IPC.
#[derive(Debug)]
pub struct BidirChild {
    /// The spawned child process.
    pub child: Child,
    /// Sender for writing to the child.
    pub tx: Sender,
    /// Receiver for reading from the child.
    pub rx: Recver,
}

impl BidirChild {
    /// Spawns a new child process with bidirectional unnamed pipes for IPC (platform-specific).
    #[cfg(unix)]
    pub fn new(command: &mut Command) -> Result<Self, SamError> {
        // Use a helper to reduce code duplication and improve clarity
        fn create_pipes() -> Result<((Sender, Recver), (Sender, Recver)), SamError> {
            let (parent_to_child_tx, parent_to_child_rx) = pipe().map_err(|e| {
                eprintln!("Pipe creation failed: {e}");
                SamError::UnknownError
            })?;
            let (child_to_parent_tx, child_to_parent_rx) = pipe().map_err(|e| {
                eprintln!("Pipe creation failed: {e}");
                SamError::UnknownError
            })?;
            Ok(((parent_to_child_tx, parent_to_child_rx), (child_to_parent_tx, child_to_parent_rx)))
        }

        let ((parent_to_child_tx, parent_to_child_rx), (child_to_parent_tx, child_to_parent_rx)) = create_pipes()?;

        let child_to_parent_tx_handle: i32 = child_to_parent_tx.into_raw_fd();
        let parent_to_child_rx_handle: i32 = parent_to_child_rx.into_raw_fd();

        let child = command
            .arg(format!("--tx={child_to_parent_tx_handle}"))
            .arg(format!("--rx={parent_to_child_rx_handle}"))
            .spawn()
            .map_err(|e| {
                eprintln!("Unable to spawn a child process: {e}");
                SamError::UnknownError
            })?;

        Ok(Self {
            child,
            tx: parent_to_child_tx,
            rx: child_to_parent_rx,
        })
    }

    #[cfg(windows)]
    pub fn new(command: &mut Command) -> Result<Self, SamError> {
        use interprocess::os::windows::unnamed_pipe::CreationOptions;
        // Helper for pipe creation and error logging
        fn create_pipes() -> Result<((Sender, Recver), (Sender, Recver)), SamError> {
            let (parent_to_child_tx, parent_to_child_rx) = CreationOptions::default()
                .inheritable(true)
                .build()
                .map_err(|e| {
                    eprintln!("Pipe creation failed: {e}");
                    SamError::UnknownError
                })?;
            let (child_to_parent_tx, child_to_parent_rx) = CreationOptions::default()
                .inheritable(true)
                .build()
                .map_err(|e| {
                    eprintln!("Pipe creation failed: {e}");
                    SamError::UnknownError
                })?;
            Ok(((parent_to_child_tx, parent_to_child_rx), (child_to_parent_tx, child_to_parent_rx)))
        }

        let ((parent_to_child_tx, parent_to_child_rx), (child_to_parent_tx, child_to_parent_rx)) = create_pipes()?;

        let child_to_parent_tx_handle: OwnedHandle = child_to_parent_tx.into();
        let parent_to_child_rx_handle: OwnedHandle = parent_to_child_rx.into();

        let child = command
            .arg(format!(
                "--tx={}",
                child_to_parent_tx_handle.as_raw_handle() as usize
            ))
            .arg(format!(
                "--rx={}",
                parent_to_child_rx_handle.as_raw_handle() as usize
            ))
            .spawn()
            .map_err(|e| {
                eprintln!("Unable to spawn a child process: {e}");
                SamError::UnknownError
            })?;

        // Drop handles not needed by parent
        drop(parent_to_child_rx_handle);
        drop(child_to_parent_tx_handle);

        Ok(Self {
            child,
            tx: parent_to_child_tx,
            rx: child_to_parent_rx,
        })
    }
}
