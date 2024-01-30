use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct KernelPackageValidator {
    name: String,
    version: String,
    signature: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OSValidator {
    distributor: String,
    description: String,
    version: String,
    codename: String,
    kernel: KernelPackageValidator,
    loaded_modules: Vec<String>,
}
