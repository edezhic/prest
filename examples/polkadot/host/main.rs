
use runtime::*;
use clap::Parser;
use sc_cli::SubstrateCli;
use sc_executor::WasmExecutor;

#[derive(Debug, clap::Parser)]
pub struct Cli {
    #[clap(flatten)]
    pub run: sc_cli::RunCmd,
}
impl SubstrateCli for Cli {
    fn impl_name() -> String {
        "Minimal Polkadot SDK Node".into()
    }
    fn impl_version() -> String {
        env!("CARGO_PKG_VERSION").into()
    }
    fn description() -> String {
        env!("CARGO_PKG_DESCRIPTION").into()
    }
    fn author() -> String {
        env!("CARGO_PKG_AUTHORS").into()
    }
    fn support_url() -> String {
        "https://github.com/your-username/minimal-polkadot-sdk-node/issues".into()
    }
    fn copyright_start_year() -> i32 {
        2024
    }
    fn load_spec(&self, _: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
        Ok(Box::new(crate::chain_spec::development_config()?))
    }
}

pub mod chain_spec {
    use super::*;
    use sc_service::{ChainType, GenericChainSpec};
    use serde::Serialize;
    use sp_runtime::BuildStorage;
    use sc_chain_spec::GetExtension;
    use std::any::{Any, TypeId};

    #[derive(Default, Serialize, Clone)]
    pub struct RuntimeGenesisConfig;

    impl GetExtension for RuntimeGenesisConfig {
        fn get_any(&self, _: TypeId) -> &dyn Any {
            &()
        }
        fn get_any_mut(&mut self, _: TypeId) -> &mut dyn Any {
            todo!()
        }
    }

    impl BuildStorage for RuntimeGenesisConfig {
        fn assimilate_storage(
            &self,
            storage: &mut sp_core::storage::Storage,
        ) -> Result<(), String> {
            frame_system::GenesisConfig::<Runtime>::default().assimilate_storage(storage)
        }
    }

    pub fn development_config() -> Result<GenericChainSpec<RuntimeGenesisConfig>, String> {
        const WASM_BINARY: &[u8] = &[];//include_bytes!("../target/wasm32-unknown-unknown/release/polkadot.wasm");
        let chain_spec = GenericChainSpec::builder(WASM_BINARY, Default::default())
            .with_name("Development")
            .with_id("dev")
            .with_chain_type(ChainType::Development)
            .build();
        Ok(chain_spec)
    }
}

fn main() -> sc_cli::Result<()> {
    let cli = Cli::parse();
    let runner = cli.create_runner(&cli.run)?;
    runner.run_node_until_exit(|config| async move {
        let executor: WasmExecutor<sp_io::SubstrateHostFunctions> =
            sc_executor::WasmExecutor::builder()
                .build();
        let (_, _, _, task_manager) =
            sc_service::new_full_parts::<Block, RuntimeApi, _>(&config, None, executor)?;
        Ok(task_manager)
    })
}
