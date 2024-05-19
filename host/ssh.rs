use crate::*;

use russh::*;
use russh_keys::*;
use semver::Version;
use tokio::io::AsyncWriteExt;

const APPS_PATH: &str = "/home";

impl SshSession {
    pub async fn connect(addrs: &str, user: &str, password: &str) -> Result<Self> {
        let config = client::Config {
            inactivity_timeout: Some(std::time::Duration::from_secs(5)),
            ..<_>::default()
        };

        let config = Arc::new(config);

        let mut session = client::connect(config, addrs, Client {}).await?;
        let auth_res = session.authenticate_password(user, password).await?;

        if auth_res {
            info!("SSH connected to {addrs}");
        } else {
            return Err(host::Error::Anyhow(anyhow!("SSH authentication failed")));
        }

        Ok(Self { session })
    }

    pub async fn upload(&mut self, path: &str, name: &str, version: &Version) -> Result<String> {
        let binary =
            std::fs::read(path).map_err(|e| anyhow!("failed to find the built binary: {e}"))?;

        let remote_filename = format!("{name}_v{version}");
        let remote_path = format!("{APPS_PATH}/{remote_filename}");

        let channel = self
            .session
            .channel_open_session()
            .await
            .map_err(|e| anyhow!("failed to open ssh channel: {e}"))?;

        channel
            .request_subsystem(true, "sftp")
            .await
            .map_err(|e| anyhow!("failed to request sftp subsystem: {e}"))?;

        let sftp = russh_sftp::client::SftpSession::new(channel.into_stream())
            .await
            .map_err(|e| anyhow!("failed to initialize sftp session: {e}"))?;

        let mut file = sftp
            .create(&remote_path)
            .await
            .map_err(|e| anyhow!("failed to open the remote file: {e}"))?;

        file.write_all(&binary)
            .await
            .map_err(|e| anyhow!("failed to write into the remote file: {e}"))?;

        file.sync_all()
            .await
            .map_err(|e| anyhow!("failed to sync the remote binary: {e}"))?;

        self.call(&format!("chmod +x {}", &remote_path))
            .await
            .map_err(|e| anyhow!("failed to make remote binary executable: {e}"))?;

        let _ = sftp.close().await;

        info!("Uploaded {remote_filename}");

        Ok(remote_path)
    }

    pub async fn call(&mut self, command: &str) -> Result<()> {
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
        Ok(())
    }

    pub async fn close(&mut self) -> Result<()> {
        self.session
            .disconnect(Disconnect::ByApplication, "", "English")
            .await?;
        Ok(())
    }
}

pub struct SshSession {
    session: client::Handle<Client>,
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
