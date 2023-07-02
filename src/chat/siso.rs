use core::fmt::{Display, Formatter};

use anyhow::{Error, Result};
use async_openai::types::{
  ChatCompletionRequestMessage, CreateChatCompletionRequest, Role, Stop,
};
use async_trait::async_trait;
use log::{debug, error};
use tokio::time::timeout;

use crate::{
  chat::ChatModelParams, keys::Keys, policies::Policies,
  utils::get_openai_client, RequestHandler, ResponseType,
};

#[derive(Clone)]
pub struct ChatSisoRequest {
  pub system_prompt: String,
  pub user_prompt:   String,
  pub model_params:  ChatModelParams,
}

impl ChatSisoRequest {
  pub fn new(
    system_prompt: String,
    user_prompt: String,
    model_params: ChatModelParams,
  ) -> Self {
    Self {
      system_prompt,
      user_prompt,
      model_params,
    }
  }
}

pub struct ChatSisoResponse(String);

impl Display for ChatSisoResponse {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl From<ChatSisoResponse> for String {
  fn from(response: ChatSisoResponse) -> Self {
    response.0
  }
}

impl ResponseType for ChatSisoResponse {}

#[async_trait]
impl RequestHandler for ChatSisoRequest {
  type Res = ChatSisoResponse;
  async fn send(
    &self,
    policies: Policies,
    keys: Keys,
    id: u64,
  ) -> Result<Self::Res> {
    debug!("starting request {}", id);
    let client = get_openai_client(&keys);
    let mut retry_policy = policies.retry_policy;

    // continue trying until we get a response or we reach max retry
    loop {
      let request = build_inner_request(self.clone());
      let timer = timing::start();
      let timeout_duration = std::cmp::min(
        std::time::Duration::from_secs_f32(
          10.0
            * ((self.model_params.max_tokens as f32
              + (self.system_prompt.len() + self.user_prompt.len()) as f32
                / 4.0) as f32
              / 512.0),
        ),
        policies.timeout_policy.timeout,
      );
      let response =
        timeout(timeout_duration.clone(), client.chat().create(request)).await;

      // if we timed out, we need to check if we should retry
      let response = match response {
        Ok(response) => response,
        Err(err) => {
          debug!(
            "request {} timed out after {}s",
            id,
            timeout_duration.as_secs_f32()
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
      let completion =
        response.choices[0].message.clone().content.ok_or_else(|| {
          Error::msg("response.choices[0].message.content is None")
        })?;

      return Ok(ChatSisoResponse(completion));
    }
  }
}

fn build_inner_request(params: ChatSisoRequest) -> CreateChatCompletionRequest {
  CreateChatCompletionRequest {
    model: params.model_params.model,
    messages: vec![
      ChatCompletionRequestMessage {
        role:          Role::System,
        content:       Some(params.system_prompt),
        name:          None,
        function_call: None,
      },
      ChatCompletionRequestMessage {
        role:          Role::User,
        content:       Some(params.user_prompt),
        name:          None,
        function_call: None,
      },
    ],
    temperature: Some(params.model_params.temperature),
    top_p: Some(params.model_params.top_p),
    max_tokens: Some(params.model_params.max_tokens as u16),
    presence_penalty: Some(params.model_params.presence_penalty),
    frequency_penalty: Some(params.model_params.frequency_penalty),
    stop: if params.model_params.stop.is_empty() {
      None
    } else if params.model_params.stop.len() == 1 {
      Some(Stop::String(params.model_params.stop[0].clone()))
    } else {
      Some(Stop::StringArray(params.model_params.stop.clone()))
    },
    ..Default::default()
  }
}
