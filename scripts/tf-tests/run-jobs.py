#!/usr/bin/env python3

import re
import argparse
import subprocess
import json
import logging
from time import sleep
from pathlib import Path
from typing import Dict, Optional


def parse_arguments():
    parser = argparse.ArgumentParser(
        description="Submit jobs and monitor their status on Testflinger."
    )
    parser.add_argument(
        "--machines-file",
        type=Path,
        default="machines.txt",
        help="Path to the file with machines Canonical IDs",
    )
    parser.add_argument(
        "--template-file",
        type=Path,
        default="tf-job.yaml",
        help="Path to Testflinger job template file",
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default="job_outputs",
        help="Path to job outputs directory",
    )
    parser.add_argument(
        "--poll-interval",
        type=int,
        default=30,
        help="Time delay between status checks in seconds",
    )
    group = parser.add_mutually_exclusive_group()
    group.add_argument(
        "--send-jobs",
        action="store_true",
        help="Only submit jobs without monitoring their statuses",
    )
    group.add_argument(
        "--check-status",
        action="store_true",
        help="Only check job statuses without submitting new jobs",
    )
    return parser.parse_args()


# Job Submission Functions
def load_canonical_ids(filename: Path) -> list:
    """Reads the Canonical IDs from a file."""
    with open(filename, "r", encoding="utf8") as file:
        return file.read().strip().splitlines()


def create_job_yaml(template_file: Path, canonical_id: str) -> str:
    """Creates a modified job YAML for a specific Canonical ID."""
    with open(template_file, "r", encoding="utf8") as file:
        job_yaml = file.read()
    return job_yaml.replace("$CANONICAL_ID", canonical_id)


def write_temp_job_file(job_yaml: str, output_dir: Path, canonical_id: str) -> Path:
    """Writes the modified job YAML to a temporary file."""
    temp_job_file = output_dir / f"{canonical_id}_tf-job.yaml"
    temp_job_file.write_text(job_yaml)
    return temp_job_file


def submit_job(temp_job_file: Path, canonical_id: str) -> Optional[str]:
    """Submits the job and returns the job UUID."""
    try:
        result = subprocess.run(
            ["testflinger", "submit", str(temp_job_file)],
            capture_output=True,
            text=True,
            check=True,
        )
        for line in result.stdout.splitlines():
            if line.startswith("job_id:"):
                return line.split(": ")[1].strip()
        logging.warning("Failed to retrieve job_id for %s", canonical_id)
    except subprocess.CalledProcessError as e:
        logging.error("Error submitting job for %s: %s", canonical_id, e.stderr)
    return None


def save_job_uuid(job_uuid: str, output_dir: Path, canonical_id: str):
    """Creates a directory for the Canonical ID and saves the job UUID."""
    id_dir = output_dir / canonical_id
    id_dir.mkdir(exist_ok=True)
    (id_dir / "tf_job_id.txt").write_text(job_uuid)
    logging.info("Job submitted for %s with job_id: %s", canonical_id, job_uuid)


def submit_all_jobs(
    machines_file: Path, template_file: Path, output_dir: Path
) -> Dict[str, Path]:
    """Submit all jobs for the given machines."""
    canonical_ids = load_canonical_ids(machines_file)
    job_ids = {}

    for canonical_id in canonical_ids:
        job_yaml = create_job_yaml(template_file, canonical_id)
        temp_job_file = write_temp_job_file(job_yaml, output_dir, canonical_id)
        job_uuid = submit_job(temp_job_file, canonical_id)

        if job_uuid:
            save_job_uuid(job_uuid, output_dir, canonical_id)
            job_ids[job_uuid] = output_dir / canonical_id

        temp_job_file.unlink()  # Clean up temporary YAML file

    return job_ids


# Job Monitoring Functions
def load_jobs(output_dir: Path) -> Dict[str, Path]:
    """Load job IDs and directories from the job outputs directory."""
    jobs = {}
    for id_dir in output_dir.iterdir():
        if id_dir.is_dir():
            job_id_file = id_dir / "tf_job_id.txt"
            if job_id_file.exists():
                job_id = job_id_file.read_text().strip()
                jobs[job_id] = id_dir
    return jobs


def check_job_status(job_id: str) -> Optional[str]:
    """Check the status of a job by its job ID."""
    try:
        result = subprocess.run(
            ["testflinger", "status", job_id],
            capture_output=True,
            text=True,
            check=True,
        )
        return result.stdout.strip()
    except subprocess.CalledProcessError as e:
        logging.error("Error checking status for job %s: %s", job_id, e.stderr)
    return None


def extract_status_from_output(test_output):
    """Extracts the status value from the test output JSON."""
    match_ = re.search(r'"status":\s*"([^"]+)"', test_output)
    return match_.group(1) if match_ else "Unknown"


def retrieve_job_results(job_id: str, id_dir: Path):
    """Retrieve and save the results of a completed job."""
    try:
        results_result = subprocess.run(
            ["testflinger-cli", "results", job_id],
            capture_output=True,
            text=True,
            check=True,
        )
        results_data = json.loads(results_result.stdout)
        test_output = results_data.get("test_output", "")
        (id_dir / "output.txt").write_text(test_output)
        status = extract_status_from_output(test_output)
        (id_dir / "hw_status.txt").write_text(status)
        logging.info("Results and status saved for job %s in %s", job_id, id_dir)
    except (subprocess.CalledProcessError, json.JSONDecodeError) as e:
        logging.error("Error fetching results for job %s: %s", job_id, str(e))


def monitor_jobs(remaining_jobs: Dict[str, Path], poll_interval: int):
    """Monitor jobs until all are completed, fetching results as jobs finish."""
    while remaining_jobs:
        for job_id, id_dir in list(remaining_jobs.items()):
            job_status = check_job_status(job_id)
            logging.info(
                "Status for job %s (Canonical ID: %s): %s",
                job_id,
                id_dir.name,
                job_status,
            )
            if job_status == "cancelled":
                logging.error("The job %s got cancelled.", job_id)
                del remaining_jobs[job_id]
            if job_status == "complete":
                retrieve_job_results(job_id, id_dir)
                del remaining_jobs[job_id]

        if remaining_jobs:
            logging.info("Waiting %d seconds before checking again...", poll_interval)
            sleep(poll_interval)

    logging.info("All jobs complete and results retrieved.")


def main():
    logging.basicConfig(level=logging.INFO, format="%(message)s")
    args = parse_arguments()
    args.output_dir.mkdir(exist_ok=True)

    if args.send_jobs:
        submit_all_jobs(args.machines_file, args.template_file, args.output_dir)
    elif args.check_status:
        remaining_jobs = load_jobs(args.output_dir)
        monitor_jobs(remaining_jobs, args.poll_interval)
    else:
        job_ids = submit_all_jobs(
            args.machines_file, args.template_file, args.output_dir
        )
        monitor_jobs(job_ids, args.poll_interval)


if __name__ == "__main__":
    main()
