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
use interprocess::unnamed_pipe::{Recver, Sender};
use std::env;
#[cfg(unix)]
use std::os::fd::FromRawFd;
#[cfg(windows)]
use std::os::windows::io::{FromRawHandle, RawHandle};
use std::process::exit;

#[derive(Debug)]
pub struct Arguments {
    pub is_orchestrator: bool,
    pub is_app: u32,
    pub rx: Option<Recver>,
    pub tx: Option<Sender>,
}
pub fn parse_arguments() -> Arguments {
    let mut args = Arguments {
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

        match arg.as_str() {
            "--orchestrator" => {
                args.is_orchestrator = true;
                continue;
            }
            _ => unsafe {
                let split: Vec<&str> = arg.split("=").collect();
                if split.len() != 2 {
                    continue;
                }

                let key = split[0];
                let value = split[1];

                if value.len() == 0 {
                    continue;
                }

                if key == "--app" {
                    args.is_app = value.parse::<u32>().unwrap();
                    continue;
                }

                #[cfg(target_os = "linux")]
                if key == "--tx" {
                    let raw_handle = value.parse::<i32>().expect("Invalid value for --tx");
                    args.tx = Some(Sender::from_raw_fd(raw_handle));
                    continue;
                }

                #[cfg(target_os = "windows")]
                if key == "--tx" {
                    let raw_handle =
                        value.parse::<usize>().expect("Invalid value for --tx") as RawHandle;
                    args.tx = Some(Sender::from_raw_handle(raw_handle));
                    continue;
                }

                #[cfg(target_os = "linux")]
                if key == "--rx" {
                    let raw_handle = value.parse::<i32>().expect("Invalid value for --rx");
                    args.rx = Some(Recver::from_raw_fd(raw_handle));
                    continue;
                }

                #[cfg(target_os = "windows")]
                if key == "--rx" {
                    let raw_handle =
                        value.parse::<usize>().expect("Invalid value for --rx") as RawHandle;
                    args.rx = Some(Recver::from_raw_handle(raw_handle));
                    continue;
                }
            },
        }
    }

    if args.tx.is_some() != args.rx.is_some() {
        eprintln!("Invalid arguments, tx and rx must be provided.");
        exit(1);
    }

    dev_println!("New process launched with arguments: {:?}", args);

    args
}
