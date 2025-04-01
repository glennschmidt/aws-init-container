use crate::config::Source;
use aws_config::SdkConfig;
use aws_sdk_secretsmanager::error::SdkError;
use aws_sdk_secretsmanager::operation::get_secret_value::GetSecretValueError;
use std::path::Path;
use std::{io, result};
use thiserror::Error;
use tokio::fs;
use tokio::io::{AsyncWriteExt, BufWriter};

pub struct Provisioner {
    sm_client: aws_sdk_secretsmanager::Client,
}

impl Provisioner {
    pub fn new(aws_config: &SdkConfig) -> Self {
        Self {
            sm_client: aws_sdk_secretsmanager::Client::new(aws_config),
        }
    }

    pub async fn provision(&self, file_name: &str, source_config: &Source) -> Result<()> {
        let path = Path::new(file_name);
        if !path.is_absolute() {
            return Err(Error::InvalidPath(String::from(file_name)));
        }

        //Create parent directories if required
        match path.parent() {
            None => {},
            Some(parent_dir) => {
                fs::create_dir_all(parent_dir).await?;
            },
        };

        //Retrieve secret value
        let result = self.sm_client.get_secret_value()
            .secret_id(&source_config.source_arn)
            .send().await?;
        let data = match result.secret_binary {
            Some(blob) => blob.into_inner(),
            None => result.secret_string.map(|b| b.into_bytes()).unwrap_or(Vec::new()),
        };
        if data.len() == 0 {
            log::warn!("Secret value is empty for {}", source_config.source_arn);
        }

        //Write file
        let mut writer = BufWriter::new(fs::File::create(path).await?);
        writer.write_all(&data).await?;
        writer.flush().await?;

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("failed to get secret value from Secrets Manager")]
    GetSecretValueError(#[from] GetSecretValueError),

    #[error("expecting an absolute destination path: {0}")]
    InvalidPath(String),
}

impl From<SdkError<GetSecretValueError>> for Error {
    fn from(value: SdkError<GetSecretValueError>) -> Self {
        Error::GetSecretValueError(value.into_service_error())
    }
}

pub type Result<T> = result::Result<T, Error>;
