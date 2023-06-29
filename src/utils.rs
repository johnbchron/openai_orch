use async_openai::{config::OpenAIConfig, Client as OpenAIClient};

use crate::keys::Keys;

pub fn get_openai_client(keys: &Keys) -> OpenAIClient<OpenAIConfig> {
  let config = OpenAIConfig::new().with_api_key(&keys.openai_api_key);
  let config = match &keys.openai_org_id {
    Some(openai_org_id) => config.with_org_id(openai_org_id),
    None => config,
  };
  OpenAIClient::<OpenAIConfig>::with_config(config)
}
