#[derive(Clone)]
pub struct Keys {
  pub openai_api_key: String,
  pub openai_org_id:  Option<String>,
}

impl Keys {
  pub fn new(openai_api_key: String, openai_org_id: Option<String>) -> Self {
    Self {
      openai_api_key,
      openai_org_id,
    }
  }

  pub fn from_env() -> Option<Self> {
    dotenv::dotenv().ok();
    let openai_api_key = std::env::var("OPENAI_API_KEY").ok()?;
    let openai_org_id = std::env::var("OPENAI_ORG_ID").ok();
    Some(Self::new(openai_api_key, openai_org_id))
  }
}
