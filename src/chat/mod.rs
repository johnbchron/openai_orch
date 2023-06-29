pub mod siso;

#[derive(Clone)]
pub struct ChatModelParams {
  model:             String,
  temperature:       f32,
  top_p:             f32,
  stop:              Vec<String>,
  max_tokens:        u64,
  frequency_penalty: f32,
  presence_penalty:  f32,
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
