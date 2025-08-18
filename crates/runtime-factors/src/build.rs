use std::path::PathBuf;

use super::{TriggerAppArgs, TriggerFactors, TriggerFactorsRuntimeConfig};

use anyhow::Context as _;
use spin_factors_executor::FactorsExecutor;
use spin_runtime_config::ResolvedRuntimeConfig;
use spin_trigger::cli::{
    FactorsConfig, InitialKvSetterHook, KeyValueDefaultStoreSummaryHook, MaxInstanceMemoryHook,
    MemoryTrackerHook, RuntimeFactorsBuilder, SqlStatementExecutorHook, SqliteDefaultStoreSummaryHook,
    StdioLoggingExecutorHooks,
};
use spin_variables_static::StaticVariablesProvider;

/// A [`RuntimeFactorsBuilder`] for [`TriggerFactors`].
pub struct FactorsBuilder;

impl RuntimeFactorsBuilder for FactorsBuilder {
    type CliArgs = TriggerAppArgs;
    type Factors = TriggerFactors;
    type RuntimeConfig = ResolvedRuntimeConfig<TriggerFactorsRuntimeConfig>;

    fn build(
        config: &FactorsConfig,
        args: &Self::CliArgs,
    ) -> anyhow::Result<(Self::Factors, Self::RuntimeConfig)> {
        let mut runtime_config = ResolvedRuntimeConfig::<TriggerFactorsRuntimeConfig>::from_file(
            config.runtime_config_file.clone().as_deref(),
            config.local_app_dir.clone().map(PathBuf::from),
            config.state_dir.clone(),
            config.log_dir.clone(),
        )?;

        let cli_static_variables = args.get_variables()?.clone();
        let cli_static_variables_provider = StaticVariablesProvider::new(cli_static_variables);

        // Insert the parsed static variables provided via cli arguments
        // into the set of variable providers with highest precedence.
        runtime_config
            .runtime_config
            .variables
            .as_mut()
            .unwrap()
            .providers
            .insert(0, Box::new(cli_static_variables_provider));

        runtime_config.summarize(config.runtime_config_file.as_deref());

        let factors = TriggerFactors::new(
            runtime_config.state_dir(),
            config.working_dir.clone(),
            args.allow_transient_write,
        )
        .context("failed to create factors")?;
        Ok((factors, runtime_config))
    }

    fn configure_app<U: Send + 'static>(
        executor: &mut FactorsExecutor<Self::Factors, U>,
        runtime_config: &Self::RuntimeConfig,
        config: &FactorsConfig,
        args: &Self::CliArgs,
    ) -> anyhow::Result<()> {
        executor.add_hooks(StdioLoggingExecutorHooks::new(
            config.follow_components.clone(),
            runtime_config.log_dir(),
            config.truncate_logs,
        ));
        executor.add_hooks(SqlStatementExecutorHook::new(
            args.sqlite_statements.clone(),
        ));
        executor.add_hooks(InitialKvSetterHook::new(args.key_values.clone()));
        executor.add_hooks(SqliteDefaultStoreSummaryHook);
        executor.add_hooks(KeyValueDefaultStoreSummaryHook);
        executor.add_hooks(MemoryTrackerHook::new());

        let max_instance_memory = args
            .max_instance_memory
            .or(runtime_config.max_instance_memory());

        // Only add the hook if a max instance memory size is specified via flag or runtime config.
        if let Some(max_instance_memory) = max_instance_memory {
            executor.add_hooks(MaxInstanceMemoryHook::new(max_instance_memory));
        }

        Ok(())
    }
}
