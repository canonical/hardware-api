# Manual integration tests using Testflinger

The purpose of these scripts is to test hardware-api client and server
on the machines that are accessible via
[Testflinger](https://github.com/canonical/testflinger). It allows us
to test the project components on multiple machines automatically.

## Requirements

These scripts use [the testflinger
snap](https://snapcraft.io/testflinger-cli), so make sure you have it
installed on your system:

```sh
sudo snap install testflinger-cli
```

Also, the following files are required to be present in this directory:

- `machines.txt`: This file lists the Canonical IDs of the machines on
  which jobs will be run. Each Canonical ID should be on a separate
  line, formatted as follows:

  ```text
  202101-28595
  202012-28526
  ...
  ```

- `tf-job.yaml`: This YAML template defines the job parameters. The
  template should include the placeholder `$CANONICAL_ID`, which will
  be replaced with each actual Canonical ID from `machines.txt` when
  jobs are submitted.  You can modify the existing `tf-job.yaml` file
  to run other commands or use a different Ubuntu distro.

- `hwctl`: The client executable. You can create it by running `cargo
  build --release` in the project root directory. You're supposed to
  build the binary on the same Ubuntu release that is specified in
  `tf-job.yaml`. The machines should also be of the same architecture
  the binary is build for.

  Then copy the created file to this directory:

  ```sh
  cp  target/release/hwctl scripts/tf-tests/
  ```

## Running the scripts

After you meel the described requirements, make sure you have access
to
[https://testflinger.canonical.com](https://testflinger.canonical.com).

These scripts are designed to work sequentially, where `run-jobs.py`
must be executed first to submit jobs and create job directories,
followed by `check-status.py` to monitor and fetch job results.

```sh
./run-jobs.py
# wait until the script is completed
./check-status.py
```

## Scripts overview

### `run-jobs.py`

This script submits jobs based on the Canonical IDs listed in machines.txt and generates directories for each ID.
How it works:

1. Reads each Canonical ID from machines.txt.
2. Replaces `$CANONICAL_ID` in `tf-job.yaml` with the actual ID.
3. Submits the job with `testflinger submit <job.yaml>`.
4. Captures the job UUID returned after submission.
5. Creates a directory for each Canonical ID in `job_outputs/` and saves
   the job UUID in a file named `tf_job_id.txt` within that directory.

It creates a directory with the following structure:

```
job_outputs/
├── 202101-28595/
│   ├── tf_job_id.txt  # Contains the job UUID
├── 202012-28526/
│   ├── tf_job_id.txt
```

Each `tf_job_id.txt` file will contain the job UUID for the respective Canonical ID.

### `check-status.py`

This script monitors the status of each job until all jobs are
completed. For each completed job, it retrieves the test results and
saves them in the corresponding directory.

How it works:

1. Reads `tf_job_id.txt` files in `job_outputs/` to get the job UUIDs.
2. Enters a loop, checking the status of each job using `testflinger status <job_id>`.
3. For jobs with status "complete", retrieves results using `testflinger-cli results <job_id> | jq -r .test_output`.
4. Saves the test output to `output.txt` within the respective Canonical ID’s directory.
5. Extracts `hwapi` status and writes it to the `hw_status.txt`
6. Continues monitoring until all jobs are completed.

Each Canonical ID directory will contain output.txt with the test results.

```
job_outputs/
├── 202101-28595/
│   ├── tf_job_id.txt
│   ├── output.txt     # Contains test output
|   ├── hw_status.txt  # hwapi status
├── 202012-28526/
│   ├── tf_job_id.txt
│   ├── output.txt
│   ├── hw_status.txt
```
