[package]
name = "my-http-server"
version = "0.2.7"
edition = "2021"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[features]
default = []
static_files = []
full = []
my-telemetry = ["dep:my-telemetry"]


[dependencies]
rust-extensions = { tag = "0.1.2", git = "https://github.com/MyJetTools/rust-extensions.git" }

my-json = { tag = "0.1.2", git = "https://github.com/MyJetTools/my-json.git" }

my-telemetry = { tag = "0.2.2", git = "https://github.com/MyJetTools/my-telemetry.git", optional = true }

url-utils = { tag = "0.1.0", git = "https://github.com/MyJetTools/url-utils.git" }

tokio = { version = "*", features = ["full"] }

lazy_static = "*"
hyper = { version = "*", features = ["full"] }
serde = { version = "*", features = ["derive"] }
serde_json = "*"
async-trait = "*"
