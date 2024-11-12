# Manual integration tests using Testflinger

The purpose of the `run-jobs.py` script is to test hardware-api client
and server on the machines that are accessible via
[Testflinger](https://github.com/canonical/testflinger). It allows us
to test the project components on multiple machines automatically.

## Requirements

These scripts use [the testflinger
snap](https://snapcraft.io/testflinger-cli), so make sure you have it
installed on your system:

```sh
sudo snap install testflinger-cli
```

Also, the following files are required to be present in this
directory:

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
  cp target/release/hwctl scripts/tf-tests/
  ```

## Running the script

After you meet the described requirements, make sure you have access
to
[https://testflinger.canonical.com](https://testflinger.canonical.com).

The script `run-jobs.py` can be used to submit jobs, monitor their
status, or both, depending on the options you provide.

```sh
../tf_test.py [options]
```

Examples:

* Submit Jobs and Monitor Statuses Sequentially: `./tf_test.py`
* Only Submit Jobs: `./tf_test.py --send-jobs`
* Only Monitor Job Statuses: `./tf_test.py --check-status`
* Custom Machines File and Poll Interval: `./tf_test.py
  --machines-file custom_machines.txt --poll-interval 60`


## Script overview

The script performs two main functions:

- Job Submission
- Job Monitoring

### Job Submission

When submitting jobs, the script:

1. Reads each Canonical ID from `machines.txt` (or the file specified
   with `--machines-file`).
2. Replaces `$CANONICAL_ID` in `tf-job.yaml` (or the file specified
   with `--template-file`) with the actual ID.
3. Submits the job with `testflinger submit <job.yaml>`.
4. Captures the job UUID returned after submission.
5. Creates a directory for each Canonical ID in `job_outputs/` (or the
   directory specified with `--output-dir`) and saves the job UUID in
   a file named `tf_job_id.txt` within that directory.

Example directory structure after job submission:

```
job_outputs/
├── 202101-28595/
│   └── tf_job_id.txt  # Contains the job UUID
├── 202012-28526/
│   └── tf_job_id.txt
```

### Job Monitoring

When monitoring jobs, the script:

1. Reads `tf_job_id.txt` files in `job_outputs/` to get the job UUIDs.
2. Enters a loop, checking the status of each job using `testflinger
   status <job_id>`.
3. For jobs with status "complete", retrieves results using
   `testflinger-cli results <job_id>`.
4. Saves the test output to `output.txt` within the respective
   Canonical ID’s directory.
5. Extracts the status field from the test output and writes it to
   `hw_status.txt`.
6. Continues monitoring until all jobs are completed.

Example directory structure after job monitoring:

```
job_outputs/
├── 202101-28595/
│   ├── tf_job_id.txt
│   ├── output.txt     # Contains test output
│   └── hw_status.txt  # Contains the hardware API status
├── 202012-28526/
│   ├── tf_job_id.txt
│   ├── output.txt
│   └── hw_status.txt
```
