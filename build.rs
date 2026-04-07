fn main() {
    // Print cargo rerun if env changes
    println!("cargo:rerun-if-env-changed=BUILD_CSS");
    println!("cargo:rerun-if-env-changed=NODE_ENV");

    // Check if we should build CSS (default to true for development)
    let build_css = std::env::var("BUILD_CSS").unwrap_or_else(|_| "true".to_string());
    if build_css == "false" {
        return;
    }

    // Run npm build:css
    let status = std::process::Command::new("npm")
        .arg("run")
        .arg("build:css")
        .status()
        .expect("Failed to execute npm");

    if !status.success() {
        panic!("Tailwind CSS build failed");
    }
}
