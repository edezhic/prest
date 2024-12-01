use crate::*;

use chrono::{DateTime, NaiveDateTime, Utc};
use russh::*;
use russh_keys::*;
use semver::Version;
use std::time::Duration;
use tokio::{io::AsyncWriteExt, sync::RwLock, time::sleep};

const APPS_PATH: &str = "/home";
const DEPLOY_PREFIX: &str = "prest__";
const DEPLOY_DATETIME: &str = "%Y-%m-%d_%H:%M:%S";

state!(REMOTE: Option<RemoteHost> = async { RemoteHost::try_connect().await? });

pub(crate) async fn upload_and_activate(binary_path: &str) -> Result {
    let Some(remote) = &*REMOTE else {
        return Err(e!("No connection to the remote host"));
    };

    let deployment = DeploymentInfo::new();
    let package = deployment.package();

    info!("Initiated remote update for {package}");
    let mut conn = remote.conn().await?;

    info!("Uploading the binary");
    remote.set_state(DeploymentState::Uploading).await;
    conn.upload(binary_path, &deployment).await?;
    info!("Upload finished successfully");

    match conn.find_current_deployment(&deployment.pkg_name).await? {
        Some(p) => {
            let pid = p.pid.unwrap();
            conn.kill_process(pid).await?;
            while conn.check_process(pid).await? {
                info!("Stopping current deployment...");
                conn.kill_process(pid).await?;
                sleep(Duration::from_millis(1000)).await;
            }
            info!("Stopped current process")
        }
        None => warn!("No current deployment found"),
    }

    conn.activate_deployment(&deployment).await?;
    info!("Started new {package} process");

    remote.sync_deployments().await?;

    OK
}

pub(crate) struct RemoteHost {
    pub addr: String,
    pub user: String,
    pub pass: String,
    pub deployments: RwLock<Vec<DeploymentInfo>>,
    pub state: RwLock<DeploymentState>,
}

impl RemoteHost {
    pub async fn try_connect() -> Result<Option<Self>> {
        if *IS_REMOTE {
            return Ok(None);
        }

        let (addr, user, pass) = match (
            env::var("SSH_ADDR").ok(),
            env::var("SSH_USER").ok(),
            env::var("SSH_PASSWORD").ok(),
        ) {
            (Some(addr), Some(user), Some(password)) => {
                SshSession::connect(&addr, &user, &password).await?;
                info!("Connected to the remote host at {addr}");
                (addr, user, password)
            }
            _ => return Ok(None),
        };

        let state = DeploymentState::Idle;

        let host = RemoteHost {
            addr,
            user,
            pass,
            deployments: RwLock::default(),
            state: RwLock::new(state),
        };

        host.sync_deployments().await?;

        Ok(Some(host))
    }

    pub async fn conn(&self) -> Result<SshSession> {
        Ok(SshSession::connect(&self.addr, &self.user, &self.pass).await?)
    }

    pub async fn state(&self) -> DeploymentState {
        *self.state.read().await
    }

    pub async fn sync_deployments(&self) -> Result {
        let mut conn = self.conn().await?;
        let mut deployments = conn.list_deployments().await?;

        let active = conn
            .find_current_deployment(&APP_CONFIG.check().name)
            .await?;

        if let Some(active) = active {
            if let Some(deployment) = deployments.iter_mut().find(|d| {
                d.pkg_name == active.pkg_name
                    && d.version == active.version
                    && d.datetime == active.datetime
            }) {
                deployment.pid = active.pid;
            }
        }

        *self.deployments.write().await = deployments;
        OK
    }

    pub async fn set_state(&self, new_state: DeploymentState) {
        *self.state.write().await = new_state;
    }

    pub async fn ready_to_deploy(&self) -> bool {
        matches!(
            self.state().await,
            DeploymentState::Idle | DeploymentState::Success | DeploymentState::Failure
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DeploymentInfo {
    pub pid: Option<u32>,
    pub pkg_name: String,
    pub version: Version,
    pub datetime: DateTime<Utc>,
}

impl DeploymentInfo {
    pub fn new() -> Self {
        let cfg = APP_CONFIG.check();
        let pkg_name = &cfg.name;
        let version = &cfg.version;

        DeploymentInfo {
            pid: None,
            pkg_name: pkg_name.to_owned(),
            version: version.clone(),
            datetime: Utc::now(),
        }
    }

    pub fn from(command: &str, pid: Option<&str>) -> Result<Self> {
        let Some(binary) = command.strip_prefix(&format!("{APPS_PATH}/{DEPLOY_PREFIX}")) else {
            return Err(e!("Unexpected process command: {command}"));
        };

        let mut values = binary.split("__");

        let Some(pkg_name) = values.next().map(|s| s.to_owned()) else {
            return Err(e!("Expected process package name: {command}"));
        };

        let Some(raw_version) = values.next() else {
            return Err(e!("Expected process package version: {command}"));
        };
        let version = raw_version.parse::<semver::Version>().somehow()?;

        let Some(raw_datetime) = values.next() else {
            return Err(e!("Expected process package datetime: {command}"));
        };
        let datetime = NaiveDateTime::parse_from_str(raw_datetime, DEPLOY_DATETIME)
            .somehow()?
            .and_utc();

        let pid = match pid {
            Some(s) => Some(
                s.trim()
                    .parse::<u32>()
                    .map_err(|e| e!("Failed to parse process ID: {e}"))?,
            ),
            None => None,
        };

        Ok(Self {
            pid,
            pkg_name,
            version,
            datetime,
        })
    }

    pub fn path(&self) -> String {
        let datetime = self.datetime.format(DEPLOY_DATETIME).to_string();
        let remote_filename = format!(
            "{DEPLOY_PREFIX}{}__{}__{datetime}",
            self.pkg_name, self.version
        );
        format!("{APPS_PATH}/{remote_filename}")
    }

    pub fn package(&self) -> String {
        format!("{} v{}", self.pkg_name, self.version)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum DeploymentState {
    Idle,
    Building,
    Uploading,
    Success,
    Failure,
}

struct Client {}
#[async_trait]
impl client::Handler for Client {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &key::PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

pub(crate) struct SshSession {
    session: client::Handle<Client>,
}

impl SshSession {
    pub async fn connect(addr: &str, user: &str, password: &str) -> Result<Self> {
        let config = client::Config {
            inactivity_timeout: Some(std::time::Duration::from_secs(5)),
            ..<_>::default()
        };

        let config = Arc::new(config);

        let mut session = client::connect(config, addr, Client {}).await?;
        let auth_res = session.authenticate_password(user, password).await?;

        if !auth_res {
            return Err(e!("SSH authentication failed"));
        }

        Ok(Self { session })
    }

    pub async fn upload(&mut self, local_path: &str, deployment: &DeploymentInfo) -> Result {
        let binary = std::fs::read(local_path).somehow()?;

        let remote_path = deployment.path();

        let channel = self
            .session
            .channel_open_session()
            .await
            .map_err(|e| e!("failed to open ssh channel: {e}"))?;

        channel
            .request_subsystem(true, "sftp")
            .await
            .map_err(|e| e!("failed to request sftp subsystem: {e}"))?;

        let sftp = russh_sftp::client::SftpSession::new(channel.into_stream())
            .await
            .map_err(|e| e!("failed to initialize sftp session: {e}"))?;

        let mut file = sftp
            .create(&remote_path)
            .await
            .map_err(|e| e!("failed to open the remote file: {e}"))?;

        file.write_all(&binary)
            .await
            .map_err(|e| e!("failed to write into the remote file: {e}"))?;

        file.sync_all()
            .await
            .map_err(|e| e!("failed to sync the remote binary: {e}"))?;

        self.call(&format!("chmod +x {}", &remote_path))
            .await
            .map_err(|e| e!("failed to make the remote binary executable: {e}"))?;

        let _ = sftp.close().await;
        OK
    }

    pub async fn call(&mut self, command: &str) -> Result {
        let mut channel = self.session.channel_open_session().await?;
        channel.exec(true, command).await?;

        let mut stdout = io::stdout();

        loop {
            let Some(msg) = channel.wait().await else {
                break;
            };
            if let ChannelMsg::Data { ref data } = msg {
                tokio::io::AsyncWriteExt::write_all(&mut stdout, data).await?;
                tokio::io::AsyncWriteExt::flush(&mut stdout).await?;
            }
        }
        OK
    }

    #[allow(dead_code)]
    pub async fn close(&mut self) -> Result {
        self.session
            .disconnect(Disconnect::ByApplication, "", "English")
            .await?;
        OK
    }

    pub async fn find_prest_processes(&mut self) -> Result<Vec<DeploymentInfo>> {
        let mut channel = self.session.channel_open_session().await?;
        channel
            .exec(true, format!(r#"pgrep -fa "{DEPLOY_PREFIX}""#))
            .await?;

        let mut output = Vec::new();
        while let Some(msg) = channel.wait().await {
            if let ChannelMsg::Data { ref data } = msg {
                output.extend_from_slice(data);
            }
        }

        Ok(String::from_utf8_lossy(&output)
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(2, ' ').collect();
                match parts[..] {
                    [pid, cmd] => match DeploymentInfo::from(cmd, Some(pid)) {
                        Ok(p) => Some(p),
                        Err(e) => {
                            warn!("Invalid process info: {e}");
                            None
                        }
                    },
                    _ => None,
                }
            })
            .collect())
    }

    pub async fn find_current_deployment(
        &mut self,
        package: &String,
    ) -> Result<Option<DeploymentInfo>> {
        Ok(self
            .find_prest_processes()
            .await?
            .into_iter()
            .find(|p| &p.pkg_name == package))
    }

    pub async fn kill_process(&mut self, pid: u32) -> Result {
        self.call(&format!("kill -SIGTERM {pid}")).await?;
        OK
    }

    pub async fn check_process(&mut self, pid: u32) -> Result<bool> {
        Ok(self
            .find_prest_processes()
            .await?
            .iter()
            .find(|p| p.pid.is_some() && p.pid.unwrap() == pid)
            .is_some())
    }

    pub async fn list_deployments(&mut self) -> Result<Vec<DeploymentInfo>> {
        let mut channel = self.session.channel_open_session().await?;
        channel
            .exec(true, format!(r#"ls -1 {APPS_PATH}/{DEPLOY_PREFIX}*"#))
            .await?;

        let mut output = Vec::new();
        while let Some(msg) = channel.wait().await {
            if let ChannelMsg::Data { ref data } = msg {
                output.extend_from_slice(data);
            }
        }

        Ok(String::from_utf8_lossy(&output)
            .lines()
            .filter_map(|line| match DeploymentInfo::from(line, None) {
                Ok(p) => Some(p),
                Err(e) => {
                    warn!("Invalid deployment file: {e}");
                    None
                }
            })
            .collect())
    }

    pub async fn activate_deployment(&mut self, deployment: &DeploymentInfo) -> Result {
        self.call(&format!("DEPLOYED_TO_REMOTE=true {}", deployment.path()))
            .await?;
        OK
    }

    pub async fn delete_deployment(&mut self, deployment: &DeploymentInfo) -> Result {
        self.call(&format!("rm {}", deployment.path())).await?;
        OK
    }
}
