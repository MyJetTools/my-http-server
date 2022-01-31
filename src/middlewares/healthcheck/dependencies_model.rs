use std::collections::HashMap;
use serde::Serialize;

#[derive(Serialize)]
pub struct Dependencies {
    #[serde(rename = "DependenciesMap")]
    pub list: HashMap<String, String> 
}

impl Dependencies {
    pub fn new() -> Self {
        Self { 
            list: HashMap::new()
        }
    }

    pub fn from(hs: HashMap<String, String>) -> Self {
        Self { 
            list: hs
        }
    }

    pub fn add(&mut self, package_name: &str, package_version: &str) {
        self.list.insert(package_name.to_string(), package_version.to_string());
    }

    pub fn get_package_version(&self, key: &str) -> Option<&String> {
        self.list.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_dependencies() {
        // create new object
        let ver = "1.0.0";

        let mut dependencies = Dependencies::new();
        dependencies.add("foo", ver);

        // act
        let version = dependencies.get_package_version("foo").unwrap();

        // assert
        assert_eq!(version, ver);
    }
}