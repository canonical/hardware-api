# Test Data Directory

This example data serves two purposes:

- provides a comprehensive set of example data against which unit
  and/or component tests can be run

- serve as a means of demonstrating to interested parties exactly what
  data the library collects and is transmitted to the hardware API
  server. All fields should be non-uniquely indentifiable to ensure
  that they tracking or tracing of machines interacting with service
  is possible

Currently, the project uses test data of the following machines:

- NVIDIA DGX-Station:
  [ubuntu.com/certified/201711-25989](https://ubuntu.com/certified/201711-25989)
- Lenovo ThinkStation P620:
  [ubuntu.com/certified/202203-30052](https://ubuntu.com/certified/202203-30052)
- Dell XPS 13 9340:
  [ubuntu.com/certified/202405-34051](https://ubuntu.com/certified/202405-34051)
- Raspberry Pi 4 Model B 8 GB:
  [ubuntu.com/certified/202004-27865](https://ubuntu.com/certified/202004-27865)

## Structure Overview

The `test_data` directory has the following structure:

```
├── <arch>
│   ├── <device_model>
│   │   ├── <hardware_related_data>
│   │   └── expected.json
```

Key Files:

- `cpuinfo_max_freq` — Contains maximum CPU frequency data. Can be
  fetched from `/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_min_freq`
- `DMI` — DMI table data, can be retrieved from the
  `/sys/firmware/dmi/tables/` directory.
- `smbios_entry_point` — SMBIOS entry point details. Is located in the
  `/sys/firmware/dmi/tables/` directory.
- `version` — Kernel version file (`/proc/version`).
- `expected.json` — Expected request body that should be constructed.
- `cpuinfo` — CPU information, see `/proc/cpuinfo`. If used only for
  non-amd64 machine.
- `device-tree/` (dir) — arm64 specific directory with system
  information. See `/proc/device-tree`.

## Adding New Test Cases

1. Create a new directory under `amd64/` or `generic/` based on your
   test requirements.
2. Populate it with the necessary files (`cpuinfo_max_freq`, `DMI`,
   etc.). **Make sure you replace all the sensitive data like serial
   numbers with zeroes.** You can use the
   [`sed`](https://man7.org/linux/man-pages/man1/sed.1.html) tool for
   it.
3. Add `expected.json` with anticipated values to validate against.
   Replace the values that you want to use variable with placeholders, for example:
   ```
   "os": {
    "codename": "$CODENAME",
    "distributor": "Ubuntu",
    "kernel": {
      "loaded_modules": $KERNEL_MODULES,
      "name": "Linux",
      "signature": null,
      "version": "$KERNEL_VERSION"
    },
    "version": "$RELEASE"
   }
   ```

   In this example, we replace values in the quotes with strings
   defined in the test cases, and `$KERNEL_MODULES` gets replaced
   with a dict. This is how the test case is defined (see tests in the
   [models/request_validators.rs](../src/models/request_validators.rs) for
   more):

   ```rust
   #[test_case(
       "amd64/thinkstation_p620",  // dir name
       "focal",  // Ubuntu release codename
       "20.04",  // The release itself
       "5.15.0-125-generic",  // kernel version from the {dir_name}/version file
       &["xt_tcpudp", "nft_chain_nat"];  // kernel modules
       "focal_thinkstation"  // test case name
   )]
   ```

4. If you're adding a test case for a certified machine, update this
   document to include link to the machine on
   [ubuntu.com/certified](https://ubuntu.com/certified).
