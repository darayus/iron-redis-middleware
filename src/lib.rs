extern crate iron;

extern crate r2d2;
extern crate r2d2_redis;
extern crate redis;

use iron::prelude::*;
use iron::{typemap, BeforeMiddleware};

use std::sync::Arc;
use r2d2_redis::RedisConnectionManager;

/// Iron middleware that allows for redis connections within requests.
pub struct RedisMiddleware {
  /// A pool of redis connections that are shared between requests.
  pub pool: Arc<r2d2::Pool<r2d2_redis::RedisConnectionManager>>,
}

pub struct Value(Arc<r2d2::Pool<r2d2_redis::RedisConnectionManager>>);

impl typemap::Key for RedisMiddleware { type Value = Value; }

impl RedisMiddleware {

  /// Creates a new pooled connection to the given redis server
  pub fn new<R: redis::IntoConnectionInfo>(params: R) -> Result<RedisMiddleware, redis::RedisError> {
    let config = r2d2::Config::builder()
        .error_handler(Box::new(r2d2::LoggingErrorHandler))
        .build();
    let manager = try!(RedisConnectionManager::new(params));
    let pool = Arc::new(r2d2::Pool::new(config, manager).unwrap());
    return Ok(RedisMiddleware {
      pool: pool,
    });
  }
}

impl BeforeMiddleware for RedisMiddleware {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<RedisMiddleware>(Value(self.pool.clone()));
        Ok(())
    }
}

/// Adds a method to requests to get a redis connection.
pub trait RedisReqExt {
  /// Returns a pooled connection to the redis database. The connection is returned to
  /// the pool when the pooled connection is dropped.
  fn redis_conn(&self) -> r2d2::PooledConnection<r2d2_redis::RedisConnectionManager>;
}

impl<'a, 'b> RedisReqExt for Request<'a, 'b> {
  fn redis_conn(&self) -> r2d2::PooledConnection<r2d2_redis::RedisConnectionManager> {
    let poll_value = self.extensions.get::<RedisMiddleware>().unwrap();
    let &Value(ref poll) = poll_value;

    return poll.get().unwrap();
  }
}
