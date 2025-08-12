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

use crate::dev_println;
use gtk::gio::ApplicationCommandLine;
use gtk::prelude::ApplicationCommandLineExt;
use interprocess::unnamed_pipe::{Recver, Sender};
#[cfg(unix)]
use std::os::fd::FromRawFd;
#[cfg(windows)]
use std::os::windows::io::{FromRawHandle, RawHandle};
use std::{cell::Cell, env, process::exit, rc::Rc};

/// Parsed command-line arguments for orchestrator/app mode.
#[derive(Debug)]
pub struct CliArguments {
    pub is_orchestrator: bool,
    pub is_app: u32,
    pub rx: Option<Recver>,
    pub tx: Option<Sender>,
}

/// Parsed arguments for GUI mode.
#[derive(Debug)]
pub struct GuiArguments {
    pub auto_open: Rc<Cell<u32>>,
}

/// Parses command-line arguments for orchestrator/app mode.
pub fn parse_cli_arguments() -> CliArguments {
    let mut args = CliArguments {
        is_orchestrator: false,
        is_app: 0,
        rx: None,
        tx: None,
    };

    for (index, arg) in env::args().enumerate() {
        if index == 0 {
            // Self binary name
            continue;
        }

        if arg == "--orchestrator" {
            args.is_orchestrator = true;
            continue;
        }

        // Parse --key=value arguments
        if let Some((key, value)) = arg.split_once('=') {
            if value.is_empty() {
                continue;
            }

            if key == "--app" {
                match value.parse::<u32>() {
                    Ok(val) => args.is_app = val,
                    Err(_) => {
                        eprintln!("Invalid value for --app: {}", value);
                        exit(1);
                    }
                }
                continue;
            }

            #[cfg(target_os = "linux")]
            if key == "--tx" {
                match value.parse::<i32>() {
                    Ok(raw_handle) => args.tx = Some(unsafe { Sender::from_raw_fd(raw_handle) }),
                    Err(_) => {
                        eprintln!("Invalid value for --tx: {}", value);
                        exit(1);
                    }
                }
                continue;
            }

            #[cfg(target_os = "windows")]
            if key == "--tx" {
                match value.parse::<usize>() {
                    Ok(raw_handle) => args.tx = Some(unsafe { Sender::from_raw_handle(raw_handle as RawHandle) }),
                    Err(_) => {
                        eprintln!("Invalid value for --tx: {}", value);
                        exit(1);
                    }
                }
                continue;
            }

            #[cfg(target_os = "linux")]
            if key == "--rx" {
                match value.parse::<i32>() {
                    Ok(raw_handle) => args.rx = Some(unsafe { Recver::from_raw_fd(raw_handle) }),
                    Err(_) => {
                        eprintln!("Invalid value for --rx: {}", value);
                        exit(1);
                    }
                }
                continue;
            }

            #[cfg(target_os = "windows")]
            if key == "--rx" {
                match value.parse::<usize>() {
                    Ok(raw_handle) => args.rx = Some(unsafe { Recver::from_raw_handle(raw_handle as RawHandle) }),
                    Err(_) => {
                        eprintln!("Invalid value for --rx: {}", value);
                        exit(1);
                    }
                }
                continue;
            }
        }
    }

    if args.tx.is_some() != args.rx.is_some() {
        eprintln!("Invalid arguments, tx and rx must be provided.");
        exit(1);
    }

    dev_println!("New process launched with arguments: {:?}", args);
    args
}

/// Parses GUI arguments from a GTK ApplicationCommandLine.
pub fn parse_gui_arguments(cmd_line: &ApplicationCommandLine) -> GuiArguments {
    let arguments = cmd_line.arguments();
    let args = GuiArguments {
        auto_open: Rc::new(Cell::new(0)),
    };

    for arg in arguments.iter().skip(1) {
        // Skip the first argument (program name)
        if let Some(arg_str) = arg.to_str() {
            if let Some(value_str) = arg_str.strip_prefix("--auto-open=") {
                match value_str.parse::<u32>() {
                    Ok(value) => {
                        args.auto_open.set(value);
                        println!("Parsed --auto-open value: {}", value);
                    }
                    Err(_) => {
                        eprintln!("Error: Invalid value for --auto-open: {}", value_str);
                    }
                }
            }
        }
    }

    args
}
