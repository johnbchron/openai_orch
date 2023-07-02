pub mod chat;
pub mod keys;
pub mod policies;
pub mod utils;

use std::{any::Any, collections::HashMap, marker::PhantomData, sync::Arc};

use anyhow::{Error, Result};
use async_trait::async_trait;
use tokio::sync::{mpsc, Mutex, Semaphore};

use crate::{keys::Keys, policies::Policies};

pub trait ResponseType: 'static + Send {}

#[async_trait]
pub trait RequestHandler {
  type Res: ResponseType;
  async fn send(
    &self,
    policies: Policies,
    keys: Keys,
    id: u64,
  ) -> Result<Self::Res>;
}

#[derive(Clone)]

pub struct RequestID<R: ResponseType> {
  id:      u64,
  _marker: PhantomData<R>,
}

type ResponseReceiver = mpsc::Receiver<Result<Box<dyn Any + Send>>>;

#[derive(Clone)]
pub struct Orchestrator {
  requests:  Arc<Mutex<HashMap<u64, ResponseReceiver>>>,
  semaphore: Arc<Semaphore>,
  policies:  Policies,
  keys:      Keys,
}

impl Orchestrator {
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

  pub async fn add_request<R, Req>(&self, request: Req) -> RequestID<R>
  where
    Req: RequestHandler<Res = R> + Send + Sync + 'static,
    R: ResponseType,
  {
    let id = rand::random::<u64>();
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

  pub async fn get_response<R: ResponseType>(
    &self,
    request_id: RequestID<R>,
  ) -> Result<R> {
    // let mut requests = self.requests.lock().await;
    // let mut rx = requests
    //   .remove(&request_id.id)
    //   .ok_or_else(|| Error::msg("No response receiver found"))?;
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
