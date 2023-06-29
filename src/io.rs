
use anyhow::Result;

use crate::{Keys, Policies, RequestHandler, ResponseType};

pub struct RequestA;
pub struct ResponseA;

pub struct RequestB;
pub struct ResponseB;

impl ResponseType for ResponseA {}
impl ResponseType for ResponseB {}

impl RequestHandler for RequestA {
  type Res = ResponseA;
  fn send(&self, policies: Policies, keys: Keys) -> Result<Self::Res> {
    Ok(ResponseA)
  }
}

impl RequestHandler for RequestB {
  type Res = ResponseB;
  fn send(&self, policies: Policies, keys: Keys) -> Result<Self::Res> {
    Ok(ResponseB)
  }
}
