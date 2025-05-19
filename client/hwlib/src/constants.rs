/* Copyright 2024 Canonical Ltd.
 *
 * This program is free software: you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public License
 * version 3, as published by the Free Software Foundation.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * Written by:
 *        Nadzeya Hutsko <nadzeya.hutsko@canonical.com>
 */

pub const CERT_STATUS_ENDPOINT: &str = "/v1/certification/status";

pub const CPU_MAX_FREQ_FILE_PATH: &str = "/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq";
pub const PROC_CPUINFO_FILE_PATH: &str = "/proc/cpuinfo";
pub const PROC_DEVICE_TREE_DIR_PATH: &str = "/proc/device-tree/";
pub const PROC_VERSION_FILE_PATH: &str = "/proc/version";

pub const ARCH: &str = "/usr/bin/arch";
pub const LSB_RELEASE: &str = "/usr/bin/lsb_release";
pub const LSMOD: &str = "/usr/bin/lsmod";
