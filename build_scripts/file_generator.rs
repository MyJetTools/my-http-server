use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

use cargo_lock::Lockfile;

pub const FILE_NAME: &str = "repo_deps.rs";

fn generate_deps_list() -> HashMap<String, String> {
    
    let lockfile = Lockfile::load("Cargo.lock").unwrap();

    let mut hs = HashMap::new();

    for pkg in lockfile.packages {
        let ver = format!("{}", pkg.version);
        hs.insert(pkg.name.as_str().to_string(), ver);
    }

    hs
}

pub fn generate_file_with_deps_list() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join(FILE_NAME);

    let packages = generate_deps_list(); 

    let mut inserts = vec![];
    for (pkg_name, pkg_version) in packages {
        inserts.push(
            format!("hs.insert(\"{}\".into(), \"{}\".into());", pkg_name, pkg_version));
    }

    let content = format!(r###"
        use std::collections::HashMap;
        
        pub fn return_project_deps() -> HashMap<String, String> {{
            let mut hs = HashMap::new();
            {}
        
            hs
        }}
    
"###, inserts.join("\n"));

    fs::write(
        &dest_path,
        content
    ).unwrap();

    println!("cargo:rerun-if-changed=cargo.toml");
}
