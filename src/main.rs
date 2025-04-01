mod cli;
mod config;
mod provisioner;

use crate::cli::Args;
use crate::config::Config;
use crate::provisioner::Provisioner;

use anyhow::{Context, Error};
use aws_config::sts::AssumeRoleProviderBuilder;
use aws_config::{BehaviorVersion, Region};
use clap::Parser;
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::env;
use std::rc::Rc;
use std::str::FromStr;
use tokio::task::LocalSet;

const APP: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const CONFIG_ENV_VAR: &str = "CONTAINER_INIT_CONF";

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    //Parse configuration
    let args = Args::parse();
    let config = match &args.config {
        None => {
            let yaml = env::var(CONFIG_ENV_VAR)
                .context(format!("Missing environment variable {}", CONFIG_ENV_VAR))?;
            Config::from_yaml_str(&yaml)
                .context(format!("Invalid YAML in environment variable {}", CONFIG_ENV_VAR))?
        }
        Some(config_file) => {
            Config::from_file(config_file)
                .context(format!("Error loading configuration from file {}", config_file))?
        }
    };

    //Init logging
    let log_level = LevelFilter::from_str(&config.log_level)
        .context(format!("Unrecognized log_level {}", config.log_level))?;
    SimpleLogger::new().with_level(log_level).init()?;
    log::info!("{} v{}", APP, VERSION);

    //Configure the AWS SDK
    let mut aws_config = aws_config::defaults(BehaviorVersion::v2025_01_17());
    if config.aws.region.is_some() {
        aws_config = aws_config.region(Region::new(config.aws.region.unwrap()));
    }
    if config.aws.assume_role_arn.is_some() {
        let arn = config.aws.assume_role_arn.as_ref().unwrap();
        let mut assume_role = AssumeRoleProviderBuilder::new(arn);
        if config.aws.assume_role_external_id.is_some() {
            assume_role = assume_role.external_id(config.aws.assume_role_external_id.unwrap());
        }
        log::info!("Assuming role {}", arn);
        aws_config = aws_config.credentials_provider(assume_role.build().await);
    }
    let aws_config = aws_config.load().await;

    //Retrieve data and provision files as configured
    let tasks = LocalSet::new();
    let provisioner = Rc::new(Provisioner::new(&aws_config));
    for (file_name, source) in config.files {
        let provisioner = provisioner.clone();
        tasks.spawn_local(async move {
            let result = provisioner.provision(&file_name, &source).await;
            match result.context(format!("Unable to provision file {}", file_name)) {
                Ok(_) => log::info!("Provisioned file {}", file_name),
                Err(err) => log::error!("{:#}", err),
            }
        });
    }
    tasks.await;

    log::info!("Finished.");
    Ok(())
}
