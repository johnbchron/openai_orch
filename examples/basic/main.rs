use std::time::Duration;

use openai_orch::{
  chat::siso::ChatSisoRequest, keys::Keys, policies::{Policies, ConcurrencyPolicy, TimeoutPolicy}, Orchestrator,
};

#[tokio::main]
async fn main() {
  let orch = Orchestrator::new(Policies {
    concurrency_policy: ConcurrencyPolicy::new(50),
    timeout_policy: TimeoutPolicy::new(Duration::from_secs(15)),
    ..Default::default()
  }, Keys::from_env().unwrap());

  let mut request = ChatSisoRequest {
    id:            0,
    system_prompt: "You are a helpful assistant.".to_string(),
    user_prompt:   "Hi".to_string(),
    model_params:  Default::default(),
  };

  let mut request_handles = vec![];
  for i in 0..100 {
    request.id = i;
    request_handles.push(orch.add_request(request.clone()).await);
  }

  let mut response_handles = vec![];
  for handle in request_handles {
    response_handles.push(tokio::spawn({
      let orch = orch.clone();
      async move {
        let response = orch.get_response(handle).await.unwrap();
        println!("{}", response);
      }
    }));
  }
  futures::future::join_all(response_handles).await;
}
