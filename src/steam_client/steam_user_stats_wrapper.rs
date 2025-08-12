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

//! Provides a safe Rust abstraction over the `ISteamUserStats` FFI interface.
use crate::steam_client::steam_user_stats_vtable::ISteamUserStats;
use crate::steam_client::steamworks_types::{CSteamID, SteamAPICall_t};
use crate::steam_client::wrapper_types::SteamClientError;
use std::sync::Arc;

/// Safe wrapper for the `ISteamUserStats` interface.
#[derive(Debug, Clone)]
pub struct SteamUserStats {
    inner: Arc<SteamUserStatsInner>,
}

#[derive(Debug)]
struct SteamUserStatsInner {
    ptr: *mut ISteamUserStats,
}

impl SteamUserStats {
    /// Creates a new `SteamUserStats` instance from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be valid and remain valid for the lifetime of the `SteamUserStats` instance.
    pub unsafe fn from_raw(ptr: *mut ISteamUserStats) -> Self {
        Self {
            inner: Arc::new(SteamUserStatsInner { ptr }),
        }
    }

    /// Gets whether an achievement is unlocked and its unlock time.
    pub fn get_achievement_and_unlock_time(
        &self,
        achievement_name: &str,
    ) -> Result<(bool, u32), SteamClientError> {
        unsafe {
            let vtable = (*self.inner.ptr)
                .vtable
                .as_ref()
                .ok_or(SteamClientError::NullVtable)?;

            let mut achieved = false;
            let mut unlock_time = 0u32;
            let c_achievement_name = std::ffi::CString::new(achievement_name)
                .map_err(|_| SteamClientError::UnknownError)?;

            let success = (vtable.get_achievement_and_unlock_time)(
                self.inner.ptr,
                c_achievement_name.as_ptr(),
                &mut achieved,
                &mut unlock_time,
            );

            if success {
                Ok((achieved, unlock_time))
            } else {
                Err(SteamClientError::UnknownError)
            }
        }
    }

    /// Unlocks an achievement.
    pub fn set_achievement(&self, achievement_name: &str) -> Result<(), SteamClientError> {
        unsafe {
            let vtable = (*self.inner.ptr)
                .vtable
                .as_ref()
                .ok_or(SteamClientError::NullVtable)?;

            let c_achievement_name = std::ffi::CString::new(achievement_name)
                .map_err(|_| SteamClientError::UnknownError)?;

            let success = (vtable.set_achievement)(self.inner.ptr, c_achievement_name.as_ptr());

            if success {
                Ok(())
            } else {
                Err(SteamClientError::UnknownError)
            }
        }
    }

    /// Clears (locks) an achievement.
    pub fn clear_achievement(&self, achievement_name: &str) -> Result<(), SteamClientError> {
        unsafe {
            let vtable = (*self.inner.ptr)
                .vtable
                .as_ref()
                .ok_or(SteamClientError::NullVtable)?;

            let c_achievement_name = std::ffi::CString::new(achievement_name)
                .map_err(|_| SteamClientError::UnknownError)?;

            let success = (vtable.clear_achievement)(self.inner.ptr, c_achievement_name.as_ptr());

            if success {
                Ok(())
            } else {
                Err(SteamClientError::UnknownError)
            }
        }
    }

    /// Gets a 32-bit integer stat value.
    pub fn get_stat_i32(&self, stat_name: &str) -> Result<i32, SteamClientError> {
        unsafe {
            let vtable = (*self.inner.ptr)
                .vtable
                .as_ref()
                .ok_or(SteamClientError::NullVtable)?;

            let c_stat_name =
                std::ffi::CString::new(stat_name).map_err(|_| SteamClientError::UnknownError)?;
            let mut stat_value = 0i32;

            let success =
                (vtable.get_stat_int32)(self.inner.ptr, c_stat_name.as_ptr(), &mut stat_value);

            if !success {
                return Err(SteamClientError::UnknownError);
            }

            Ok(stat_value)
        }
    }

    /// Gets a floating-point stat value.
    pub fn get_stat_float(&self, stat_name: &str) -> Result<f32, SteamClientError> {
        unsafe {
            let vtable = (*self.inner.ptr)
                .vtable
                .as_ref()
                .ok_or(SteamClientError::NullVtable)?;

            let c_stat_name =
                std::ffi::CString::new(stat_name).map_err(|_| SteamClientError::UnknownError)?;
            let mut stat_value = 0f32;

            let success =
                (vtable.get_stat_float)(self.inner.ptr, c_stat_name.as_ptr(), &mut stat_value);

            if !success {
                return Err(SteamClientError::UnknownError);
            }

            Ok(stat_value)
        }
    }

    /// Sets a 32-bit integer stat value.
    pub fn set_stat_i32(&self, stat_name: &str, stat_value: i32) -> Result<i32, SteamClientError> {
        unsafe {
            let vtable = (*self.inner.ptr)
                .vtable
                .as_ref()
                .ok_or(SteamClientError::NullVtable)?;

            let c_stat_name =
                std::ffi::CString::new(stat_name).map_err(|_| SteamClientError::UnknownError)?;

            let success = (vtable.set_stat_int32)(self.inner.ptr, c_stat_name.as_ptr(), stat_value);

            if !success {
                return Err(SteamClientError::UnknownError);
            }

            Ok(stat_value)
        }
    }

    /// Sets a floating-point stat value.
    pub fn set_stat_float(
        &self,
        stat_name: &str,
        stat_value: f32,
    ) -> Result<f32, SteamClientError> {
        unsafe {
            let vtable = (*self.inner.ptr)
                .vtable
                .as_ref()
                .ok_or(SteamClientError::NullVtable)?;

            let c_stat_name =
                std::ffi::CString::new(stat_name).map_err(|_| SteamClientError::UnknownError)?;

            let success = (vtable.set_stat_float)(self.inner.ptr, c_stat_name.as_ptr(), stat_value);

            if !success {
                return Err(SteamClientError::UnknownError);
            }

            Ok(stat_value)
        }
    }

    /// Requests global achievement percentages.
    pub fn request_global_achievement_percentages(
        &self,
    ) -> Result<SteamAPICall_t, SteamClientError> {
        unsafe {
            let vtable = (*self.inner.ptr)
                .vtable
                .as_ref()
                .ok_or(SteamClientError::NullVtable)?;

            let res = (vtable.request_global_achievement_percentages)(self.inner.ptr);

            if res == 0 {
                return Err(SteamClientError::UnknownError);
            }

            Ok(res)
        }
    }

    /// Requests user stats for a given Steam ID.
    pub fn request_user_stats(
        &self,
        steam_id: CSteamID,
    ) -> Result<SteamAPICall_t, SteamClientError> {
        unsafe {
            let vtable = (*self.inner.ptr)
                .vtable
                .as_ref()
                .ok_or(SteamClientError::NullVtable)?;

            let res = (vtable.request_user_stats)(self.inner.ptr, steam_id);

            if res == 0 {
                return Err(SteamClientError::UnknownError);
            }

            Ok(res)
        }
    }

    /// Stores the current stats on Steam.
    pub fn store_stats(&self) -> Result<bool, SteamClientError> {
        unsafe {
            let vtable = (*self.inner.ptr)
                .vtable
                .as_ref()
                .ok_or(SteamClientError::NullVtable)?;

            let res = (vtable.store_stats)(self.inner.ptr);

            Ok(res)
        }
    }

    /// Resets all stats, optionally including achievements.
    pub fn reset_all_stats(&self, achievements_too: bool) -> Result<bool, SteamClientError> {
        unsafe {
            let vtable = (*self.inner.ptr)
                .vtable
                .as_ref()
                .ok_or(SteamClientError::NullVtable)?;

            let success = (vtable.reset_all_stats)(self.inner.ptr, achievements_too);

            Ok(success)
        }
    }

    /// Gets the achieved percent for a given achievement.
    pub fn get_achievement_achieved_percent(
        &self,
        achievement_name: &str,
    ) -> Result<f32, SteamClientError> {
        unsafe {
            let vtable = (*self.inner.ptr)
                .vtable
                .as_ref()
                .ok_or(SteamClientError::NullVtable)?;

            let c_achievement_name = std::ffi::CString::new(achievement_name)
                .map_err(|_| SteamClientError::UnknownError)?;
            let mut achieved_percent = 0f32;

            let success = (vtable.get_achievement_achieved_percent)(
                self.inner.ptr,
                c_achievement_name.as_ptr(),
                &mut achieved_percent,
            );

            if !success {
                return Err(SteamClientError::UnknownError);
            }

            Ok(achieved_percent)
        }
    }
}
