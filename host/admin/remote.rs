use super::super::remote::*;
use crate::*;

pub(crate) fn routes() -> Router {
    route("/state", get(remote_view))
        .route("/deploy", get(start_deploy))
        .route("/activate", patch(activate_deployment))
        .route("/deactivate", patch(deactivate_deployment))
        .route("/delete", delete(delete_deployment))
}

pub(crate) async fn remote_view() -> Markup {
    use DeploymentState::*;
    let Some(remote) = &*REMOTE else {
        return html!();
    };

    let state = remote.state().await;
    let mut deployments = remote.deployments.read().await.clone();
    deployments.sort_by(|a, b| b.datetime.cmp(&a.datetime));

    let running = deployments.iter().find(|d| d.pid.is_some()).is_some();
    let deploying = matches!(state, Building | Uploading);

    let message = match state {
        Idle => match running {
            true => "Update",
            false => "Deploy",
        },
        Building => "Building...",
        Uploading => "Uploading...",
        Failure => "Failed. Retry?",
        Success => "Deployed!",
    };

    html!(span trigger="load delay:1s" get="/admin/remote/state" into="this" swap-full {
        $"font-bold text-lg" {"Remote host"}
        $"flex w-full" {
            $"w-1/4 flex items-center justify-center" {
                @if deploying {b{(message)}}
                @else {button $"mt-2 mb-4 w-32 rounded-lg bg-stone-700 hover:bg-stone-600 text-gray-300" get="/admin/remote/deploy" swap-none {(message)}}
            }

            $"w-3/4" { @for deployment in &deployments {
                @let active = deployment.pid.is_some();
                $"flex my-2 items-center font-mono" vals=(json!(deployment)) swap-none {
                    @if active {(active_svg())}
                    @else {$"w-8"{}}
                    b{(deployment.pkg_name)}
                    "(v"(deployment.version)")"
                    " at "(deployment.datetime)
                    @if active {button patch="/admin/remote/deactivate" {(deactivate_svg())}}
                    @else {
                        button patch="/admin/remote/activate" {(activate_svg())}
                        button delete="/admin/remote/delete" {(delete_svg())}
                    }
                }
            }}
        }

    })
}

pub(crate) async fn start_deploy() -> impl IntoResponse {
    let Some(remote) = &*REMOTE else {
        return Err(e!("No remote connection"));
    };

    if !remote.ready_to_deploy().await {
        return Err(e!("Not ready to start deploy"));
    }

    info!("Initiated deployment");
    remote.set_state(DeploymentState::Building).await;

    RT.try_once(async {
        if let Ok(Ok(binary_path)) = tokio::task::spawn_blocking(build_linux_binary).await {
            if let Err(e) = upload_and_activate(&binary_path).await {
                remote.set_state(DeploymentState::Failure).await;
                error!("Failed to update the server: {e}");
            } else {
                remote.sync_deployments().await?;
                remote.set_state(DeploymentState::Success).await;
            }
        } else {
            remote.sync_deployments().await?;
            remote.set_state(DeploymentState::Failure).await;
        }
        OK
    });

    OK
}

pub(crate) async fn activate_deployment(Vals(deployment): Vals<DeploymentInfo>) -> Result {
    let Some(remote) = &*REMOTE else {
        return Err(e!("No remote connection"));
    };

    if let Some(d) = remote
        .deployments
        .read()
        .await
        .iter()
        .find(|d| d.pid.is_some())
    {
        remote.conn().await?.kill_process(d.pid.unwrap()).await?;
    }

    remote
        .conn()
        .await?
        .activate_deployment(&deployment)
        .await?;

    let _ = remote.sync_deployments().await;
    OK
}

pub(crate) async fn deactivate_deployment(Vals(deployment): Vals<DeploymentInfo>) -> Result {
    let Some(remote) = &*REMOTE else {
        return Err(e!("No remote connection"));
    };

    let Some(pid) = deployment.pid else {
        return Err(e!("Deployment is not active"));
    };

    remote.conn().await?.kill_process(pid).await?;

    let _ = remote.sync_deployments().await;
    OK
}

pub(crate) async fn delete_deployment(Vals(deployment): Vals<DeploymentInfo>) -> Result {
    let Some(remote) = &*REMOTE else {
        return Err(e!("No remote connection"));
    };

    remote.conn().await?.delete_deployment(&deployment).await?;

    let _ = remote.sync_deployments().await;
    OK
}

fn active_svg() -> Markup {
    html!(
        svg viewBox="0 0 24 24" $"w-8 h-8" {
            circle cx="12" cy="12" r="4" fill="#4ade80" {
                animate attributeName="opacity" values="1;0.4;1" dur="1s" repeatCount="indefinite" {}
                animate attributeName="r" values="4;5;4" dur="1s" repeatCount="indefinite" {}
            }
            circle cx="12" cy="12" r="6" fill="#4ade80" opacity="0.3" {
                animate attributeName="opacity" values="0.3;0.1;0.3" dur="1s" repeatCount="indefinite" {}
                animate attributeName="r" values="6;7;6" dur="1s" repeatCount="indefinite" {}
            }
        }
    )
}

fn activate_svg() -> Markup {
    html!(
        svg viewBox="0 0 24 24" $"ml-2 w-6 h-6" { path d="M2 2l16 10L2 22z" fill="#4ade80" {}}
    )
}

fn deactivate_svg() -> Markup {
    html!(
        svg viewBox="0 0 24 24" $"ml-2 w-6 h-6" { rect x="2" y="2" width="20" height="20" fill="#ef4444" {}}
    )
}

fn delete_svg() -> Markup {
    html!(
        svg viewBox="0 0 24 24" $"ml-2 w-6 h-6" {
            path d="M4 6h16L18 22H6L4 6z" fill="#ef4444" {}
            path d="M2 6h20v-2H2v2z" fill="#ef4444" {}
            path d="M9 3h6v1H9z" fill="#ef4444" {}
        }
    )
}
