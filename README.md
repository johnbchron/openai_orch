A concurrency-included Rust client for the OpenAI API.

# Overview
`openai-orch` is designed to provide a simple interface for sending requests
to OpenAI in bulk, while managing concurrency at a global level. It also
provides configurable policies to control how concurrency, timeouts, and
retries are handled.

# Usage
To use this library, create an `Orchestrator` with the desired policies and
keys. To allow a thread to use the `Orchestrator`, simply clone it. To send
a request, call `add_request` on the `Orchestrator`, and then call get_response
on the `Orchestrator` with the request ID returned by `add_request`. The
`Orchestrator` will handle concurrency automatically.

# Example

```rust
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

```
If you'd like, you can implement `OrchRequest` on your own request type.
See the `OrchRequest` trait for more information. Currently the only request
type implemented is `ChatSisoRequest`; `SISO` stands for "Single Input Single
Output".