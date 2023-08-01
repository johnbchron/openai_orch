//! Requests and responses using Embeddings models.

use anyhow::{Error, Result};
use async_openai::types::CreateEmbeddingRequest;
use async_trait::async_trait;
use log::{debug, error};
use tokio::time::timeout;

use crate::{
  keys::Keys, policies::Policies, utils::get_openai_client, OrchRequest,
  ResponseType,
};

pub const EMBEDDING_SIZE: usize = 1536;

pub struct EmbeddingRequest(pub String);
pub struct EmbeddingResponse(pub [f32; EMBEDDING_SIZE]);

impl ResponseType for EmbeddingResponse {}

#[async_trait]
impl OrchRequest for EmbeddingRequest {
  type Res = EmbeddingResponse;
  async fn send(
    &self,
    policies: Policies,
    keys: Keys,
    id: u64,
  ) -> Result<Self::Res> {
    debug!("starting request {}", id);
    let client = get_openai_client(&keys);
    let mut retry_policy = policies.retry_policy;

    let request = CreateEmbeddingRequest {
      model: "text-embedding-ada-002".to_string(),
      input: async_openai::types::EmbeddingInput::String(self.0.to_string()),
      user:  None,
    };

    // continue trying until we get a response or we reach max retry
    loop {
      let timer = timing::start();
      let response = timeout(
        policies.timeout_policy.timeout,
        client.embeddings().create(request.clone()),
      )
      .await;

      let response = match response {
        Ok(response) => response,
        Err(err) => {
          debug!(
            "request {} timed out after {}s",
            id,
            policies.timeout_policy.timeout.as_secs_f32()
          );
          if retry_policy.failed_request().await {
            continue;
          } else {
            error!("request {} reached max retry", id);
            return Err(Error::new(err).context("reached max retry"));
          }
        }
      };

      // if we got a response, we need to check if it's an error
      let response = match response {
        Ok(response) => response,
        Err(err) => {
          if retry_policy.failed_request().await {
            continue;
          } else {
            return Err(Error::new(err).context("reached max retry"));
          }
        }
      };

      debug!(
        "got response for {} in {}",
        id,
        timer.elapsed().as_secs_f32()
      );
      let embedding = response
        .data
        .first()
        .ok_or(Error::msg("response.data is empty"))?
        .embedding
        .clone();

      return Ok(EmbeddingResponse(embedding.as_slice().try_into()?));
    }
  }
}
