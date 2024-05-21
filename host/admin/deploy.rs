use crate::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum DeploymentState {
    Idle,
    Building,
    Connecting,
    Uploading,
    Success,
    Failure,
    Already,
}

pub(crate) static DEPLOY: Lazy<std::sync::RwLock<DeploymentState>> = Lazy::new(|| {
    std::sync::RwLock::new(match env::var("DEPLOYED").is_ok() {
        true => DeploymentState::Already,
        false => DeploymentState::Idle,
    })
});

/// Interface for the [`DEPLOY`]
pub(crate) trait DeploymentUtils {
    fn state(&self) -> DeploymentState;
    fn set(&self, new_state: DeploymentState);
    fn button(&self) -> PreEscaped<String>;
    fn already(&self) -> bool;
    #[allow(dead_code)]
    fn ready(&self) -> bool;
}

impl DeploymentUtils for std::sync::RwLock<DeploymentState> {
    fn state(&self) -> DeploymentState {
        *self.read().unwrap()
    }
    fn set(&self, new_state: DeploymentState) {
        *self.write().unwrap() = new_state;
    }
    fn already(&self) -> bool {
        DeploymentState::Already == *self.read().unwrap()
    }
    fn ready(&self) -> bool {
        !self.already()
            && matches!(
                self.state(),
                DeploymentState::Idle | DeploymentState::Success | DeploymentState::Failure
            )
    }
    fn button(&self) -> PreEscaped<String> {
        let state = *self.read().unwrap();
        let msg = match state {
            DeploymentState::Building => "Building",
            DeploymentState::Connecting => "Connecting",
            DeploymentState::Uploading => "Uploading",
            DeploymentState::Failure => "Failed. Retry?",
            DeploymentState::Idle => "Deploy",
            DeploymentState::Success => "Deployed!",
            DeploymentState::Already => {
                return html!();
            }
        };
        let running = matches!(
            state,
            DeploymentState::Building | DeploymentState::Connecting | DeploymentState::Uploading
        );
        let trigger = match running {
            true => "load delay:1s",
            false => "click",
        };

        html!(
            button."btn btn-ghost" hx-get="/admin/deploy" hx-target="this" hx-swap="outerHTML" hx-trigger=(trigger) disabled[running]
                {(msg) @if running {span."loading loading-dots loading-xs"{}}}
        )
    }
}

pub(crate) async fn deploy() -> impl IntoResponse {
    match DEPLOY.state() {
        DeploymentState::Already => return html!(),
        DeploymentState::Idle => {
            info!("Initiated deployment");
            DEPLOY.set(DeploymentState::Building);
            RT.once(async {
                if let Ok(Ok(binary_path)) = tokio::task::spawn_blocking(build_linux_binary).await {
                    if let Err(e) = remote_update(&binary_path).await {
                        DEPLOY.set(DeploymentState::Failure);
                        error!("Failed to update the server: {e}");
                    } else {
                        DEPLOY.set(DeploymentState::Success);
                    }
                } else {
                    DEPLOY.set(DeploymentState::Failure);
                }
            });
        }
        _ => {}
    }
    DEPLOY.button()
}

pub(crate) async fn remote_update(binary_path: &str) -> Result {
    let addrs = env::var("SSH_ADDR")?;
    let user = env::var("SSH_USER")?;
    let password = env::var("SSH_PASSWORD")?;

    let cfg = APP_CONFIG.check();
    let name = &cfg.name;
    let version = &cfg.version;

    DEPLOY.set(DeploymentState::Connecting);
    info!("Initiated remote update for {name}_v{version}");
    let mut ssh = SshSession::connect(&addrs, &user, &password).await?;
    ssh.call(&format!("pkill -f {name}")).await?;
    info!("Stopped current {name} process");
    DEPLOY.set(DeploymentState::Uploading);
    let uploaded_binary = ssh.upload(binary_path, name, version).await?;
    info!("Uploaded the new {name} binary");
    ssh.call(&format!("DEPLOYED=true {uploaded_binary}"))
        .await?;
    info!("Started new {name} process");
    let _ = ssh.close().await;
    info!("Deployed {name} successfully");
    Ok(())
}
