use std::time::Duration;

use log::info;
use openai_orch::{
  chat::{siso::ChatSisoRequest, ChatModelParams},
  keys::Keys,
  policies::{ConcurrencyPolicy, Policies, TimeoutPolicy},
  Orchestrator,
};

#[tokio::main]
async fn main() {
  env_logger::init();

  let orch = Orchestrator::new(
    Policies {
      concurrency_policy: ConcurrencyPolicy::new(25),
      timeout_policy: TimeoutPolicy::new(Duration::from_secs(15)),
      ..Default::default()
    },
    Keys::from_env().unwrap(),
  );

  let mut request_handles = vec![];
  for _ in 0..100 {
    let request = ChatSisoRequest::new(
      "You are a helpful assistant.".to_string(),
      "Hi".to_string(),
      ChatModelParams::default(),
    );
    request_handles.push(orch.add_request(request.clone()).await);
  }

  let mut response_handles = vec![];
  for handle in request_handles {
    response_handles.push(tokio::spawn({
      let orch = orch.clone();
      async move {
        let response = orch.get_response(handle).await.unwrap();
        info!("{}", response);
      }
    }));
  }
  futures::future::join_all(response_handles).await;
}
