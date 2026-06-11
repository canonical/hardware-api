/* Copyright 2024 Canonical Ltd.
 *
 * This program is free software: you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public License
 * version 3, as published by the Free Software Foundation.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * Written by:
 *        Canonical Ltd <matias.piipari@canonical.com>
 *        Nadzeya Hutsko <nadzeya.hutsko@canonical.com>
 */

use anyhow::{Context, Result};
use clap::Parser;
use chrono::{DateTime, Utc};
use std::process::ExitCode;

use hwlib::{
    models::request_validators::{CertificationStatusRequest, Paths},
    models::response_validators::CertificationStatusResponse,
};
use hwlib::send_certification_status_request;

/// CLI tool to check hardware certification status.
///
/// The utility checks the Ubuntu Hardware Certification status of your
/// system. It verifies whether your exact machine model or a similar system has been
/// certified and displays which Ubuntu releases the certification covers for it.
///
/// For more information about the Ubuntu Hardware Certification Program, visit
/// the certification website: https://ubuntu.com/certified
///
/// Note that root privileges are typically required to run this command.
#[derive(Parser, Debug)]
#[command(version, about, long_about, verbatim_doc_comment)]
struct Args {
    #[arg(
        long = "server",
        env = "HW_API_URL",
        default_value = "https://hw.ubuntu.com",
        help = "API server URL"
    )]
    hw_api_url: String,
}

fn main() -> ExitCode {
    let args = Args::parse();
    match run(args.hw_api_url) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("ERROR: {e:?}");
            ExitCode::FAILURE
        }
    }
}

fn create_failure(reason: String) -> CertificationStatusResponse {
    let mut retval = CertificationStatusResponse::NotSeen {
        stale: None,
        stale_reason: Some(reason),
        last_attempt_at: None,
        checked_at: None,
        expires_at: None,
        hardware_mismatch: None,
        remote_access_enabled: None,
        source: None,
    };
    update_fields(&mut retval, true);
    return retval;
}

fn update_fields(response: &mut CertificationStatusResponse, stale: bool) {
    let current_utc: DateTime<Utc> = Utc::now();
    match response {
        CertificationStatusResponse::Certified { stale: s,
                                                 hardware_mismatch: hm,
                                                 remote_access_enabled: rae,
                                                 checked_at: ca,
                                                 last_attempt_at: laa,
                                                 expires_at: ea,
                                                 source: src, .. }
        | CertificationStatusResponse::NotSeen { stale: s,
                                                 hardware_mismatch: hm,
                                                 remote_access_enabled: rae,
                                                 checked_at: ca,
                                                 last_attempt_at: laa,
                                                 expires_at: ea,
                                                 source: src, .. }
        | CertificationStatusResponse::CertifiedImageExists { stale: s,
                                                              hardware_mismatch: hm,
                                                              remote_access_enabled: rae,
                                                              checked_at: ca,
                                                              last_attempt_at: laa,
                                                              expires_at: ea,
                                                              source: src, .. }
        | CertificationStatusResponse::RelatedCertifiedSystemExists { stale: s,
                                                                      hardware_mismatch: hm,
                                                                      remote_access_enabled: rae,
                                                                      checked_at: ca,
                                                                      last_attempt_at: laa,
                                                                      expires_at: ea,
                                                                      source: src, .. } => {
            *s = Some(stale);
            *hm = Some(false);
            *rae = Some(true);
            *laa = Some(current_utc.to_rfc3339());
            *src = Some("server".to_string());

            if !stale {
                *ca = Some(current_utc.to_rfc3339());
                *ea = Some((current_utc + chrono::Duration::days(30)).to_rfc3339());
            }
        }
    }
}

fn request(server_url: String) -> CertificationStatusResponse {
    let cert_status_request =
        CertificationStatusRequest::new(Paths::default());
    if cert_status_request.is_err() {
        return create_failure(("cannot collect system data").to_string());
    }

    let cert_status_request = cert_status_request.unwrap();
    let response = send_certification_status_request(server_url, &cert_status_request);
    if response.is_err() {
        return create_failure("cannot send certification status request".to_string());
    }

    let mut response2 = response.ok().unwrap();
    update_fields(&mut response2, false);
    response2
}

fn run(server_url: String) -> Result<()> {
    let response = request(server_url);
    let response_json =
        serde_json::to_string_pretty(&response).context("cannot serialize response as JSON")?;
    println!("{response_json}");

    let staled:bool;
    let stale_reason: String;

    match response {
          CertificationStatusResponse::Certified { stale: s, stale_reason: sr, .. }
        | CertificationStatusResponse::NotSeen { stale: s, stale_reason: sr, .. }
        | CertificationStatusResponse::CertifiedImageExists { stale: s, stale_reason: sr, .. }
        | CertificationStatusResponse::RelatedCertifiedSystemExists { stale: s, stale_reason: sr, .. } => {
            staled = s.unwrap_or(false);
            stale_reason = sr.unwrap_or("".to_string());
        }
    }

    if staled {
        return Err(anyhow::anyhow!(stale_reason));
    }

    Ok(())
}
