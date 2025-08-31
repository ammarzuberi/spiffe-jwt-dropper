use anyhow::Result;
use config::{Config, Environment};
use serde::Deserialize;
use spiffe::{JwtSvid, WorkloadApiClient};
use std::fs;
use std::path::Path;
use tracing::{error, info};

#[derive(Debug, Deserialize)]
struct AppConfig {
  jwt_aud: String,
  jwt_path: String,
  #[serde(default = "default_workload_api_path")]
  workload_api_path: String,
}

fn default_workload_api_path() -> String {
  "unix:/var/run/spire-agent/api.sock".to_string()
}

impl AppConfig {
  fn from_env() -> Result<Self> {
    let config = Config::builder()
    .add_source(Environment::default())
    .build()?;
    
    let app_config = config.try_deserialize::<AppConfig>()?;
    Ok(app_config)
  }
}

#[tokio::main]
async fn main() -> Result<()> {
  tracing_subscriber::fmt()
  .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
  .init();
  
  info!("starting SPIFFE JWT dropper");
  
  let config = match AppConfig::from_env() {
    Ok(config) => {
      info!("loaded configuration: aud={}, path={}", config.jwt_aud, config.jwt_path);
      config
    }
    Err(e) => {
      error!("failed to load configuration: {}", e);
      return Err(e);
    }
  };
  
  info!("connecting to SPIFFE Workload API");
  let mut client = match WorkloadApiClient::new_from_path(&config.workload_api_path).await {
    Ok(client) => {
      info!("successfully connected to SPIFFE Workload API");
      client
    }
    Err(e) => {
      error!("failed to connect to SPIFFE Workload API: {}", e);
      return Err(e.into());
    }
  };
  
  info!("fetching JWT SVID for audience: {}", config.jwt_aud);
  let jwt_svid: JwtSvid = match client.fetch_jwt_svid(&[&config.jwt_aud], None).await {
    Ok(jwt_svid) => {
      info!("successfully fetched JWT SVID");
      jwt_svid
    }
    Err(e) => {
      error!("failed to fetch JWT SVID: {}", e);
      return Err(e.into());
    }
  };
  
  let jwt_token = jwt_svid.token();
  info!("writing JWT to file: {}", config.jwt_path);
  
  if let Some(parent) = Path::new(&config.jwt_path).parent() {
    if !parent.exists() {
      info!("creating parent directories for: {}", parent.display());
      fs::create_dir_all(parent)?;
    }
  }
  
  match fs::write(&config.jwt_path, jwt_token) {
    Ok(_) => {
      info!("successfully wrote JWT to file: {}", config.jwt_path);
    }
    Err(e) => {
      error!("failed to write JWT to file {}: {}", config.jwt_path, e);
      return Err(e.into());
    }
  }
  
  info!("SPIFFE JWT dropper completed successfully");
  Ok(())
}
