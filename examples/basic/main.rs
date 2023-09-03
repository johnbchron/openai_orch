use openai_orch::prelude::*;

#[tokio::main]
async fn main() {
  let policies = Policies::default();
  let keys = Keys::from_env().unwrap();
  let orchestrator = Orchestrator::new(policies, keys);

  let request = ChatSisoRequest::new(
    "You are a helpful assistant.".to_string(),
    "What are you?".to_string(),
    Default::default(),
  );
  let request_id = orchestrator.add_request(request).await;

  let response = orchestrator
    .get_response::<ChatSisoResponse>(request_id)
    .await;
  println!("{}", response.unwrap());
}
