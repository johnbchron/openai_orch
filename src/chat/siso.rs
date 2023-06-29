use core::fmt::{Display, Formatter};

use anyhow::{Error, Result};
use async_openai::types::{
  ChatCompletionRequestMessage, CreateChatCompletionRequest, Role, Stop,
};
use async_trait::async_trait;
use tokio::time::timeout;

use crate::{
  chat::ChatModelParams, keys::Keys, policies::Policies,
  utils::get_openai_client, RequestHandler, ResponseType,
};

#[derive(Clone)]
pub struct ChatSisoRequest {
  pub id:            u32,
  pub system_prompt: String,
  pub user_prompt:   String,
  pub model_params:  ChatModelParams,
}

pub struct ChatSisoResponse(String);

impl Display for ChatSisoResponse {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl ResponseType for ChatSisoResponse {}

#[async_trait]
impl RequestHandler for ChatSisoRequest {
  type Res = ChatSisoResponse;
  async fn send(&self, policies: Policies, keys: Keys) -> Result<Self::Res> {
    println!("starting request {}", self.id);
    let client = get_openai_client(&keys);
    let mut retry_policy = policies.retry_policy;

    loop {
      let request = build_inner_request(self.clone());
      // start a timer for debugging
      let timer = timing::start();
      let response = timeout(
        policies.timeout_policy.timeout,
        client.chat().create(request),
      )
      .await;

      match response {
        Ok(response) => {
          println!(
            "got response for {} in {}",
            self.id,
            timer.elapsed().as_secs_f32()
          );
          match response {
            Ok(response) => {
              let completion =
                response.choices[0].message.clone().content.ok_or_else(
                  || Error::msg("response.choices[0].message.content is None"),
                )?;
              return Ok(ChatSisoResponse(completion));
            }
            Err(err) => {
              if retry_policy.failed_request().await {
                continue;
              } else {
                return Err(Error::new(err).context("reached max retry"));
              }
            }
          }
        }
        Err(err) => {
          println!(
            "request {} timed out after {}s",
            self.id,
            policies.timeout_policy.timeout.as_secs_f32()
          );
          if retry_policy.failed_request().await {
            continue;
          } else {
            return Err(Error::new(err).context("reached max retry"));
          }
        }
      }
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
