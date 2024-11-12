#!/usr/bin/env python3

import re
import subprocess
import json
from time import sleep
from pathlib import Path
import argparse


def parse_arguments():
    parser = argparse.ArgumentParser(
        description="Monitor job status and retrieve results."
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
    return parser.parse_args()


def load_jobs(output_dir):
    """Load job IDs and directories from the job outputs directory."""
    jobs = []
    for id_dir in output_dir.iterdir():
        if id_dir.is_dir():
            job_id_file = id_dir / "tf_job_id.txt"
            with open(job_id_file, "r") as file:
                job_id = file.read().strip()
            jobs.append((job_id, id_dir))
    return {job_id: id_dir for job_id, id_dir in jobs}


def check_job_status(job_id):
    """Check the status of a job by its job ID."""
    try:
        status_result = subprocess.run(
            ["testflinger", "status", job_id],
            capture_output=True,
            text=True,
            check=True,
        )
        return status_result.stdout.strip()
    except subprocess.CalledProcessError as e:
        print(f"Error checking status for job {job_id}: {e.stderr}")
        return None


def extract_status_from_output(test_output):
    """Extracts the status value from the test output JSON."""
    match_ = re.search(r'"status":\s*"([^"]+)"', test_output)
    return match_.group(1) if match_ else "Unknown"


def retrieve_job_results(job_id, id_dir):
    """Retrieve and save the results of a completed job, including extracting and saving the status."""
    try:
        results_result = subprocess.run(
            ["testflinger-cli", "results", job_id],
            capture_output=True,
            text=True,
            check=True,
        )

        results_data = json.loads(results_result.stdout)
        test_output = results_data.get("test_output", "")

        # Write test output to output.txt in the Canonical ID directory
        with open(id_dir / "output.txt", "w") as file:
            file.write(test_output)

        # Extract the "status" field from the test output and write to hw_status.txt
        status = extract_status_from_output(test_output)
        with open(id_dir / "hw_status.txt", "w") as status_file:
            status_file.write(status)

        print(
            f"Results saved for job {job_id} in {id_dir.name}/output.txt and status in hw_status.txt"
        )

    except subprocess.CalledProcessError as e:
        print(f"Error fetching results for job {job_id}: {e.stderr}")
    except json.JSONDecodeError:
        print(f"Error decoding JSON for job {job_id} results.")


def monitor_jobs(remaining_jobs, poll_interval):
    """Monitor jobs until all are completed, fetching results as jobs finish."""
    while remaining_jobs:
        for job_id, id_dir in list(remaining_jobs.items()):
            job_status = check_job_status(job_id)

            if job_status:
                print(
                    f"Status for job {job_id} (Canonical ID: {id_dir.name}): {job_status}"
                )

                # Retrieve results if the job is complete
                if job_status == "complete":
                    retrieve_job_results(job_id, id_dir)
                    del remaining_jobs[job_id]

        # Wait before the next round of checks if there are still jobs left
        if remaining_jobs:
            print(f"Waiting {poll_interval} seconds before checking again...")
            sleep(poll_interval)

    print("All jobs complete and results retrieved.")


def main():
    """Main function to load jobs and monitor their status."""
    args = parse_arguments()
    remaining_jobs = load_jobs(args.output_dir)
    monitor_jobs(remaining_jobs, args.poll_interval)


if __name__ == "__main__":
    main()
