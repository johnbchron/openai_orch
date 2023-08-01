//! Requests and responses using Chat models.

pub mod siso;

/// Parameters common to all OpenAI Chat models.
///
/// Refer to `async-openai`'s `CreateChatCompletionRequest` for exact details.
#[derive(Clone)]
pub struct ChatModelParams {
  pub model:             String,
  pub temperature:       f32,
  pub top_p:             f32,
  pub stop:              Vec<String>,
  pub max_tokens:        u64,
  pub frequency_penalty: f32,
  pub presence_penalty:  f32,
}

impl Default for ChatModelParams {
  fn default() -> Self {
    Self {
      model:             String::from("gpt-3.5-turbo"),
      temperature:       0.0,
      top_p:             1.0,
      stop:              vec![],
      max_tokens:        256,
      frequency_penalty: 0.0,
      presence_penalty:  0.0,
    }
  }
}
