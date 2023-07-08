use std::process::Command;

pub fn build(name: &str, target: Option<&str>) {
    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .args(["-p", name])
        // changing default target dir to avoid deadlock with other workspace builds
        .args(["--target-dir", &format!("target/{name}")]);

    if let Some(triple) = target {
        cmd.args(["--target", triple]);
    }
        
    if !cfg!(debug_assertions) {
        cmd.arg("--release");
    }
    
    assert!(cmd.status().unwrap().success());
}
