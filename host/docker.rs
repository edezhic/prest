use std::process::Stdio;

use crate::*;

pub static BUILDER_DOCKERFILE: &str = include_str!("Dockerfile");
pub const DOCKER_BUILDER_IMAGE: &str = "prest-builder";
pub const DOCKER_CARGO_CACHE_DIR: &str = "docker_cargo_cache";

pub fn build_linux_binary(project_path: &str, target_path: &str) -> Result<String> {
    prepare_docker_builder(target_path)?;
    let name = &APP_CONFIG.check().name;
    match std::process::Command::new("docker")
        .current_dir(project_path)
        .arg("run")
        .arg("--rm")
        .args(["--volume", &format!(".:/usr/src/")])
        .args(["--workdir", &format!("/usr/src/")])
        .args(["-e", &format!("CARGO_HOME=/usr/src/target/{DOCKER_CARGO_CACHE_DIR}")])
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
        Ok(s) if s.code().filter(|c| *c == 0).is_some() => {
            Ok(format!("{target_path}/{name}/x86_64-unknown-linux-gnu/release/{name}"))
        }
        Ok(s) => {
            Err(Error::Anyhow(anyhow!("Failed to build the linux binary: {s}")))
        }
        Err(e) => {
            error!("{e}");
            Err(Error::Anyhow(anyhow!("Failed to start the docker builder image")))
        }
    }
}

fn prepare_docker_builder(target_dir: &str) -> Result<()> {
    let dockerfile_path = &format!("{target_dir}/Dockerfile");

    if !std::process::Command::new("docker")
        .arg("image")
        .arg("inspect")
        .arg(DOCKER_BUILDER_IMAGE)
        .stdout(Stdio::null())
        .status()
        .unwrap()
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
            error!("{e}");
            return Err(Error::Anyhow(anyhow!("Failed to build the linux binary")));
        }
    }
    Ok(())
}
