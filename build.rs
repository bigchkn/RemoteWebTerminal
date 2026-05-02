use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=web/src");
    println!("cargo:rerun-if-changed=web/index.html");
    println!("cargo:rerun-if-changed=web/package.json");
    println!("cargo:rerun-if-changed=web/vite.config.ts");

    let web = Path::new("web");

    if !web.join("node_modules").exists() {
        let status = Command::new("npm")
            .args(["install"])
            .current_dir(web)
            .status()
            .expect("npm install failed — is Node.js installed?");
        assert!(status.success(), "npm install exited with {status}");
    }

    let status = Command::new("npm")
        .args(["run", "build"])
        .current_dir(web)
        .status()
        .expect("npm run build failed");
    assert!(status.success(), "npm run build exited with {status}");
}
