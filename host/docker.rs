use std::process::Stdio;

use crate::*;

static BUILDER_DOCKERFILE: &str = include_str!("Dockerfile");
const DOCKER_BUILDER_IMAGE: &str = "prest-builder";
const DOCKER_CARGO_CACHE_DIR: &str = "docker_cargo_cache";

pub(crate) fn build_linux_binary() -> Result<String> {
    let name = APP_CONFIG.name;
    let mut workspace_path = APP_CONFIG.manifest_dir.to_owned();

    // checking higher-level workspace path required for local dependencies
    let mut pb = std::path::PathBuf::from(&workspace_path);
    while pb.pop() {
        let mut potential_manifest = pb.clone();
        potential_manifest.push("Cargo.toml");
        if let Ok(manifest) = std::fs::read_to_string(&potential_manifest) {
            if manifest.contains("[workspace]") && manifest.contains(name) {
                let Some(path) = pb.to_str() else {
                    break;
                };
                workspace_path = path.to_owned();
            }
        }
    }

    let target_path = format!("{workspace_path}/target");

    prepare_docker_builder(&target_path)?;

    info!(target: "builder", "starting release build for deployment");
    match std::process::Command::new("docker")
        .current_dir(workspace_path)
        .arg("run")
        .arg("--rm")
        .args(["--volume", &format!(".:/usr/src/")])
        .args(["--workdir", &format!("/usr/src/")])
        .args([
            "-e",
            &format!("CARGO_HOME=/usr/src/target/{DOCKER_CARGO_CACHE_DIR}"),
        ])
        .arg("prest-builder")
        .args([
            "cargo",
            "build",
            "-p",
            name,
            "--release",
            "--target",
            "x86_64-unknown-linux-gnu",
            "--target-dir",
            &format!("./target/{name}"),
        ])
        .stdout(std::io::stdout())
        .status()
    {
        Ok(s) if s.code().filter(|c| *c == 0).is_some() => Ok(format!(
            "{target_path}/{name}/x86_64-unknown-linux-gnu/release/{name}"
        )),
        Ok(s) => Err(e!("Failed to build the linux binary: {s}")),
        Err(e) => {
            error!(target:"builder", "{e}");
            Err(e!("Failed to start the docker builder image"))
        }
    }
}

fn prepare_docker_builder(target_dir: &str) -> Result {
    let dockerfile_path = &format!("{target_dir}/Dockerfile");

    if !std::process::Command::new("docker")
        .arg("image")
        .arg("inspect")
        .arg(DOCKER_BUILDER_IMAGE)
        .stdout(Stdio::null())
        .status()
        .map_err(|e| e!("failed to check docker builder image: {e}"))?
        .success()
    {
        std::fs::write(dockerfile_path, BUILDER_DOCKERFILE)?;

        if let Err(e) = std::process::Command::new("docker")
            .current_dir(target_dir)
            .env("DOCKER_CLI_HINTS", "false")
            .arg("build")
            .args(["-t", DOCKER_BUILDER_IMAGE])
            .args(["-f", dockerfile_path])
            .arg(".")
            .stdout(std::io::stdout())
            .status()
        {
            error!(target:"builder", "{e}");
            return Err(e!("Failed to build the linux binary"));
        }
    }
    OK
}
