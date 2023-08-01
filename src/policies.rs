//! Policies for controlling retry, concurrency, and timeout behavior.

use tokio::time::Duration;

#[derive(Clone, Default)]
pub struct Policies {
  pub retry_policy:       RetryPolicy,
  pub concurrency_policy: ConcurrencyPolicy,
  pub timeout_policy:     TimeoutPolicy,
}

/// A policy for configuring how requests should retry when they fail.
#[derive(Clone)]
pub enum RetryPolicy {
  /// Retry the request immediately.
  Immediate {
    /// The number of retries attempted so far.
    current_retries: u32,
    /// The maximum number of retries to attempt.
    max_retries:     u32,
  },
  /// Retry the request after a delay.
  ConstantDelay {
    /// The number of retries attempted so far.
    current_retries: u32,
    /// The maximum number of retries to attempt.
    max_retries:     u32,
    /// The delay between retries.
    delay:           Duration,
  },
  /// Retry the request after a delay, with exponential backoff.
  ExponentialBackoff {
    /// The number of retries attempted so far.
    current_retries: u32,
    /// The maximum number of retries to attempt.
    max_retries:     u32,
    /// The initial delay between retries.
    initial_delay:   Duration,
    /// The maximum delay between retries.
    max_delay:       Duration,
  },
}

impl RetryPolicy {
  pub fn max_retries(&self) -> u32 {
    match self {
      RetryPolicy::Immediate { max_retries, .. } => *max_retries,
      RetryPolicy::ConstantDelay { max_retries, .. } => *max_retries,
      RetryPolicy::ExponentialBackoff { max_retries, .. } => *max_retries,
    }
  }

  /// Executes a retry policy, including incrementing the retry count and
  /// delaying if necessary.
  pub async fn failed_request(&mut self) -> bool {
    match self {
      RetryPolicy::Immediate {
        current_retries,
        max_retries,
      } => {
        if *current_retries < *max_retries {
          *current_retries += 1;
          true
        } else {
          false
        }
      }
      RetryPolicy::ConstantDelay {
        current_retries,
        max_retries,
        delay,
      } => {
        if *current_retries < *max_retries {
          *current_retries += 1;
          tokio::time::sleep(*delay).await;
          true
        } else {
          false
        }
      }
      RetryPolicy::ExponentialBackoff {
        current_retries,
        max_retries,
        initial_delay,
        max_delay,
      } => {
        if *current_retries < *max_retries {
          let delay =
            exponential_backoff(*current_retries, *initial_delay, *max_delay);
          *current_retries += 1;
          tokio::time::sleep(delay).await;
          true
        } else {
          false
        }
      }
    }
  }

  /// Returns a new retry policy that will retry immediately, with a maximum
  /// number of retries.
  pub fn immediate(max_retries: u32) -> Self {
    Self::Immediate {
      current_retries: 0,
      max_retries,
    }
  }

  /// Returns a new retry policy that will retry after a constant delay, with a
  /// maximum number of retries.
  pub fn constant_delay(max_retries: u32, delay: Duration) -> Self {
    Self::ConstantDelay {
      current_retries: 0,
      max_retries,
      delay,
    }
  }

  /// Returns a new retry policy that will retry after an exponentially
  /// increasing delay, with a maximum number of retries, and a maximum delay.
  pub fn exponential_backoff(
    max_retries: u32,
    initial_delay: Duration,
    max_delay: Duration,
  ) -> Self {
    Self::ExponentialBackoff {
      current_retries: 0,
      max_retries,
      initial_delay,
      max_delay,
    }
  }
}

impl Default for RetryPolicy {
  fn default() -> Self {
    Self::exponential_backoff(
      5,
      Duration::from_secs(1),
      Duration::from_secs(10),
    )
  }
}

fn exponential_backoff(
  current_retries: u32,
  initial_delay: Duration,
  max_delay: Duration,
) -> Duration {
  let delay = initial_delay * 2u32.pow(current_retries);
  if delay > max_delay {
    max_delay
  } else {
    delay
  }
}

/// A policy for configuring how many requests can be executed concurrently.
#[derive(Clone)]
pub struct ConcurrencyPolicy {
  pub max_concurrent_requests: usize,
}

impl ConcurrencyPolicy {
  pub fn new(n: usize) -> Self {
    Self {
      max_concurrent_requests: n,
    }
  }
}

impl Default for ConcurrencyPolicy {
  fn default() -> Self {
    Self {
      max_concurrent_requests: 10,
    }
  }
}

#[derive(Clone)]
pub struct TimeoutPolicy {
  pub timeout: Duration,
}

impl TimeoutPolicy {
  /// Returns a new timeout policy with the given timeout.
  pub fn new(timeout: Duration) -> Self {
    Self { timeout }
  }
}

impl Default for TimeoutPolicy {
  fn default() -> Self {
    Self {
      timeout: Duration::from_secs(30),
    }
  }
}
