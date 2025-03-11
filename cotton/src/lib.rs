mod cache;
mod config;
mod npm;
mod package;
mod plan;
mod progress;
mod resolve;
mod scoped_path;
mod util;

use color_eyre::eyre::Result;
use color_eyre::owo_colors::OwoColorize;
use compact_str::ToCompactString;
use itertools::Itertools;
use package::PackageMetadata;
use plan::tree_size;
use progress::{log_progress, log_verbose};
use resolve::Lockfile;
use std::collections::HashMap;
use std::time::Instant;
use tokio::fs::create_dir_all;
use tokio::fs::read_to_string;
use util::{read_package, write_json};

use crate::util::load_graph_from_lockfile;
use crate::{
    plan::{execute_plan, Plan},
    progress::PROGRESS_BAR,
};

pub const STORE_PATH: &str = "./target/.cotton/store";
pub const NM_COTTON_PATH: &str = "./node_modules/.cotton";
pub const NM_COTTON_PLAN_PATH: &str = "./node_modules/.cotton/plan.json";

#[tokio::main]
pub async fn run() -> Result<HashMap<String, String>> {
    let package = read_package().await?;

    init_storage().await?;

    let start = Instant::now();

    let plan = prepare_plan(&package).await?;
    let size = tree_size(&plan.trees);

    if matches!(verify_installation(&package, &plan).await, Ok(true)) {
        log_verbose("Packages already installed")
    } else {
        execute_plan(plan.clone()).await?;

        PROGRESS_BAR.suspend(|| {
            if size > 0 {
                println!(
                    "Installed {} packages in {}ms",
                    size.yellow(),
                    start.elapsed().as_millis().yellow()
                )
            }
        });
        write_json(NM_COTTON_PLAN_PATH, &plan).await?;
    }

    PROGRESS_BAR.finish_and_clear();

    Ok(package.exports)
}

async fn prepare_plan(package: &PackageMetadata) -> Result<Plan> {
    log_progress("Preparing");

    let mut graph = load_graph_from_lockfile().await;

    graph.append(package.iter_all(), true).await?;
    write_json("cotton.lock", Lockfile::new(graph.clone())).await?;

    log_progress("Retrieved dependency graph");

    let trees = graph.build_trees(&package.iter_all().collect_vec())?;
    log_progress(&format!("Fetched {} root deps", trees.len().yellow()));

    let plan = Plan::new(
        trees
            .iter()
            .map(|x| (x.root.name.to_compact_string(), x.clone()))
            .collect(),
    );

    log_progress(&format!(
        "Planned {} dependencies",
        plan.trees.len().yellow()
    ));

    Ok(plan)
}

async fn read_plan(path: &str) -> Result<Plan> {
    let plan = read_to_string(path).await?;
    Ok(serde_json::from_str(&plan)?)
}

async fn verify_installation(package: &PackageMetadata, plan: &Plan) -> Result<bool> {
    let installed = read_plan(NM_COTTON_PLAN_PATH).await?;

    if &installed != plan {
        return Ok(false);
    }

    Ok(installed.satisfies(package))
}

async fn init_storage() -> Result<()> {
    create_dir_all(STORE_PATH).await?;
    create_dir_all(NM_COTTON_PATH).await?;
    Ok(())
}
