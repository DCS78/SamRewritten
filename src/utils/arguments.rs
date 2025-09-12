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

#[cfg(target_os = "linux")]
fn parse_sender_fd(value: &str) -> Option<Sender> {
    value.parse::<i32>().ok().map(|fd| unsafe { Sender::from_raw_fd(fd) })
}
#[cfg(target_os = "windows")]
fn parse_sender_fd(value: &str) -> Option<Sender> {
    value.parse::<usize>().ok().map(|h| unsafe { Sender::from_raw_handle(h as RawHandle) })
}
#[cfg(target_os = "linux")]
fn parse_recver_fd(value: &str) -> Option<Recver> {
    value.parse::<i32>().ok().map(|fd| unsafe { Recver::from_raw_fd(fd) })
}
#[cfg(target_os = "windows")]
fn parse_recver_fd(value: &str) -> Option<Recver> {
    value.parse::<usize>().ok().map(|h| unsafe { Recver::from_raw_handle(h as RawHandle) })
}

/// Parses command-line arguments for orchestrator/app mode.
pub fn parse_cli_arguments() -> CliArguments {
    let mut args = CliArguments {
        is_orchestrator: false,
        is_app: 0,
        rx: None,
        tx: None,
    };

    for (_index, arg) in env::args().enumerate().skip(1) {
        match arg.as_str() {
            "--orchestrator" => {
                args.is_orchestrator = true;
            }
            _ if arg.starts_with("--app=") => {
                let value = &arg[6..];
                match value.parse::<u32>() {
                    Ok(val) => args.is_app = val,
                    Err(_) => {
                        eprintln!("Invalid value for --app: {}", value);
                        exit(1);
                    }
                }
            }
            _ if arg.starts_with("--tx=") => {
                let value = &arg[5..];
                match parse_sender_fd(value) {
                    Some(sender) => args.tx = Some(sender),
                    None => {
                        eprintln!("Invalid value for --tx: {}", value);
                        exit(1);
                    }
                }
            }
            _ if arg.starts_with("--rx=") => {
                let value = &arg[5..];
                match parse_recver_fd(value) {
                    Some(recver) => args.rx = Some(recver),
                    None => {
                        eprintln!("Invalid value for --rx: {}", value);
                        exit(1);
                    }
                }
            }
            _ => {}
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
