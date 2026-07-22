extern crate varlink_generator;

fn main() {
    println!("cargo:rerun-if-changed=bin/com.ubuntu.hwctl.varlink");
    varlink_generator::cargo_build_tosource("bin/com.ubuntu.hwctl.varlink", true);
    let os_release_file_path =
        std::env::var("OS_RELEASE_FILE_PATH").unwrap_or("/etc/os-release".to_owned());
    println!("cargo:rustc-env=OS_RELEASE_FILE_PATH={os_release_file_path}");
}
