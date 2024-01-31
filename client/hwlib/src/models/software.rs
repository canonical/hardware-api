use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct KernelPackage {
    pub name: String,
    pub version: String,
    pub signature: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OS {
    pub distributor: String,
    pub description: String,
    pub version: String,
    pub codename: String,
    pub kernel: KernelPackage,
    pub loaded_modules: Vec<String>,
}
