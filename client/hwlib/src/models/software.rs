use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct KernelPackageValidator {
    pub name: String,
    pub version: String,
    pub signature: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OSValidator {
    pub distributor: String,
    pub description: String,
    pub version: String,
    pub codename: String,
    pub kernel: KernelPackageValidator,
    pub loaded_modules: Vec<String>,
}
