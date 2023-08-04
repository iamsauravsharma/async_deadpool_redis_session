# ASYNC DEADPOOL REDIS SESSION

**Project status & info:**

|                    License                     |              Crates Version               |                 Docs                 |
| :--------------------------------------------: | :---------------------------------------: | :----------------------------------: |
| [![License: MIT][license_badge]][license_link] | [![Crate][cratesio_badge]][cratesio_link] | [![Docs][docsrs_badge]][docsrs_link] |

Rust library built for async session store for deadpool redis


## Add as dependencies

Edit Cargo.toml file to add alpha_vantage as dependencies

```toml
[dependencies]
async_deadpool_redis_session = "0.2.0"
```

## Usage

```rust
use async_deadpool_redis_session::{RedisSessionStore};
use async_session::{Session, SessionStore};
use deadpool_redis::{Config, Runtime};

// pool creation
let config = Config::from_url("redis://127.0.0.1:6379").unwrap();
let redis_pool = config.create_pool(Some(Runtime::Tokio1)).unwrap();
// store and session
let store = RedisSessionStore::new(redis_pool);
let mut session = Session::new();
session.insert("key", "value").unwrap();

let cookie_value = store.store_session(session).await.unwrap().unwrap();
let session = store.load_session(cookie_value).await.unwrap().unwrap();
assert_eq!(&session.get::<String>("key").unwrap(), "value");
```

See docs for detailed example of other functionality

[license_badge]: https://img.shields.io/github/license/iamsauravsharma/async_deadpool_redis_session.svg?style=for-the-badge
[license_link]: LICENSE
[cratesio_badge]: https://img.shields.io/crates/v/async_deadpool_redis_session.svg?style=for-the-badge
[cratesio_link]: https://crates.io/crates/async_deadpool_redis_session
[docsrs_badge]: https://img.shields.io/docsrs/async_deadpool_redis_session/latest?style=for-the-badge
[docsrs_link]: https://docs.rs/async_deadpool_redis_session
