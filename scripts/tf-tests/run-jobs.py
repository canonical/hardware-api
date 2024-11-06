#!/usr/bin/env python3

import subprocess
from pathlib import Path

MACHINES_FILE = "machines.txt"
TEMPLATE_FILE = "tf-job.yaml"
OUTPUT_DIR = Path("job_outputs")


def load_canonical_ids(filename):
    """Reads the Canonical IDs from a file."""
    with open(filename, "r") as file:
        return file.read().strip().splitlines()


def prepare_output_directory(directory):
    """Creates the main output directory if it doesn't exist."""
    directory.mkdir(exist_ok=True)


def create_job_yaml(template_file, canonical_id):
    """Creates a modified job YAML for a specific Canonical ID."""
    with open(template_file, "r") as file:
        job_yaml = file.read()
    return job_yaml.replace("$CANONICAL_ID", canonical_id)


def write_temp_job_file(job_yaml, output_dir, canonical_id):
    """Writes the modified job YAML to a temporary file."""
    temp_job_file = output_dir / f"{canonical_id}_tf-job.yaml"
    with open(temp_job_file, "w") as file:
        file.write(job_yaml)
    return temp_job_file


def submit_job(temp_job_file, canonical_id):
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
        print(f"Failed to retrieve job_id for {canonical_id}")
    except subprocess.CalledProcessError as e:
        print(f"Error submitting job for {canonical_id}: {e.stderr}")
    return None


def save_job_uuid(job_uuid, output_dir, canonical_id):
    """Creates a directory for the Canonical ID and saves the job UUID."""
    id_dir = output_dir / canonical_id
    id_dir.mkdir(exist_ok=True)
    with open(id_dir / "tf_job_id.txt", "w") as file:
        file.write(job_uuid)
    print(f"Job submitted successfully for {canonical_id} with job_id: {job_uuid}")
    print(f"TF URL: https://testflinger.canonical.com/jobs/{job_uuid}")


def clean_temp_file(temp_job_file):
    """Deletes the temporary job YAML file."""
    temp_job_file.unlink()


def main():
    """Main function to execute job submission workflow."""
    # Load canonical IDs
    canonical_ids = load_canonical_ids(MACHINES_FILE)
    prepare_output_directory(OUTPUT_DIR)

    # Submit each job
    for canonical_id in canonical_ids:
        job_yaml = create_job_yaml(TEMPLATE_FILE, canonical_id)
        temp_job_file = write_temp_job_file(job_yaml, OUTPUT_DIR, canonical_id)

        job_uuid = submit_job(temp_job_file, canonical_id)
        if job_uuid:
            save_job_uuid(job_uuid, OUTPUT_DIR, canonical_id)

        clean_temp_file(temp_job_file)


if __name__ == "__main__":
    main()
