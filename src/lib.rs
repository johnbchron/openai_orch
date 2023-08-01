//! A Rust client for the OpenAI API.
//!
//! `openai_orch` is designed to provide a simple interface for sending requests
//! to OpenAI in bulk, while managing concurrency at a global level. It also
//! provides configurable policies to control how concurrency, timeouts, and
//! retries are handled.

pub mod chat;
pub mod embed;
pub mod keys;
pub mod policies;
pub mod utils;

use std::{any::Any, collections::HashMap, marker::PhantomData, sync::Arc};

use anyhow::{Error, Result};
use async_trait::async_trait;
use tinyrand::Rand;
use tinyrand_std::thread_rand;
use tokio::sync::{mpsc, Mutex, Semaphore};

use crate::{keys::Keys, policies::Policies};

pub trait ResponseType: 'static + Send {}

/// Allows a request type to be used with the `Orchestrator`.
#[async_trait]
pub trait OrchRequest {
  /// The type of response returned by the request.
  type Res: ResponseType;
  /// Business logic of a request. Given the policies, keys, and request ID
  /// (for debugging, send the request and return the response.
  async fn send(
    &self,
    policies: Policies,
    keys: Keys,
    id: u64,
  ) -> Result<Self::Res>;
}

/// A unique identifier for a request.
#[derive(Clone)]
pub struct RequestID<R: ResponseType> {
  id:      u64,
  _marker: PhantomData<R>,
}

type ResponseReceiver = mpsc::Receiver<Result<Box<dyn Any + Send>>>;

/// The central interface for `openai_orch`. The `Orchestrator` is responsible
/// for managing the concurrency of requests and their responses.
///
/// Using the `Orchestrator` is simple:
/// 1. Create an `Orchestrator` with the desired policies and keys.
/// 2. Create a request type that implements `OrchRequest` (optional).
/// 3. Call `add_request` on the `Orchestrator` with the request handler.
/// 4. Call `get_response` on the `Orchestrator` with the request ID returned by
///    `add_request`.
///
/// The `Orchestrator` will handle the concurrency of requests and responses
/// automatically.
///
/// To use the `Orchestrator` in multiple parts of your application, you can
/// clone it. The `Orchestrator` is backed by an `Arc`, so cloning it is cheap.
///
/// ```rust
/// use openai_orch::{
///   chat::siso::{ChatSisoRequest, ChatSisoResponse},
///   keys::Keys,
///   policies::Policies,
///   Orchestrator,
/// };
/// 
/// #[tokio::main]
/// async fn main() {
///   let policies = Policies::default();
///   let keys = Keys::from_env().unwrap();
///   let orchestrator = Orchestrator::new(policies, keys);
/// 
///   let request = ChatSisoRequest::new(
///     "You are a helpful assistant.".to_string(),
///     "What are you?".to_string(),
///     Default::default(),
///   );
///   let request_id = orchestrator.add_request(request).await;
/// 
///   let response = orchestrator
///     .get_response::<ChatSisoResponse>(request_id)
///     .await;
///   println!("{}", response.unwrap());
/// }
/// ```
#[derive(Clone)]
pub struct Orchestrator {
  requests:  Arc<Mutex<HashMap<u64, ResponseReceiver>>>,
  semaphore: Arc<Semaphore>,
  policies:  Policies,
  keys:      Keys,
}

impl Orchestrator {
  /// Create a new `Orchestrator` with the given policies and keys.
  pub fn new(policies: Policies, keys: Keys) -> Self {
    Self {
      requests: Arc::new(Mutex::new(HashMap::new())),
      semaphore: Arc::new(Semaphore::new(
        policies.concurrency_policy.max_concurrent_requests,
      )),
      policies,
      keys,
    }
  }

  /// Add a request to the `Orchestrator`. Returns a request ID that can be used
  /// to get the response.
  ///
  /// Behind the scenes the `Orchestrator` will create a task for the request
  /// using the `OrchRequest`'s `send` method when the concurrency policy
  /// allows it. The result will be sent back to the `Orchestrator` using a
  /// channel which is mapped to the request ID.
  pub async fn add_request<R, Req>(&self, request: Req) -> RequestID<R>
  where
    Req: OrchRequest<Res = R> + Send + Sync + 'static,
    R: ResponseType,
  {
    let id = thread_rand().next_u64();
    let (tx, rx) = mpsc::channel(1);
    self.requests.lock().await.insert(id, rx);

    let semaphore = self.semaphore.clone();
    let policies = self.policies.clone();
    let keys = self.keys.clone();

    tokio::spawn(async move {
      let _permit = semaphore
        .acquire()
        .await
        .expect("failed to acquire semaphore; this is UB");

      let res = request
        .send(policies, keys, id)
        .await
        .map(|res| Box::new(res) as Box<dyn Any + Send>);
      let _ = tx.send(res).await;
    });

    RequestID {
      id,
      _marker: PhantomData,
    }
  }

  /// Get the response for a given request ID.
  ///
  /// This will block until the response is received.
  ///
  /// Behind the scenes, this listens on a channel for a task to send the
  /// response back to the `Orchestrator`. Once the response is received, it is
  /// returned.
  pub async fn get_response<R: ResponseType>(
    &self,
    request_id: RequestID<R>,
  ) -> Result<R> {
    let mut rx = self
      .requests
      .lock()
      .await
      .remove(&request_id.id)
      .ok_or_else(|| Error::msg("No response receiver found"))?;

    rx.recv()
      .await
      .ok_or_else(|| Error::msg("No response found"))?
      .map(|res| *res.downcast::<R>().expect("Failed to downcast response"))
  }
}
