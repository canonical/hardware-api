/* Copyright 2024 Canonical Ltd.
 * All rights reserved.
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * Written by:
 *        Canonical Ltd <matias.piipari@canonical.com>
 *        Nadzeya Hutsko <nadzeya.hutsko@canonical.com>
 */

pub mod collectors;
pub mod models;
pub mod py_bindings;

pub async fn get_certification_status(_url: &str) -> Result<serde_json::Value, reqwest::Error> {
    let _ = collectors::main::collect_info();
    let response_type = std::env::var("CERTIFICATION_STATUS")
        .unwrap_or_else(|_| "0".to_string())
        .parse::<i32>()
        .unwrap_or(0);

    let response = match response_type {
        1 => serde_json::json!("1"),
        2 => serde_json::json!("2"),
        _ => serde_json::json!("3"),
    };

    Ok(response)
}
