// Copyright (c) 2015 - 2017 Markus Kohlhase <mail@markus-kohlhase.de>

#![deny(
  missing_docs,
  missing_debug_implementations,
  missing_copy_implementations,
  trivial_casts,
  trivial_numeric_casts,
  unsafe_code,
  unused_import_braces,
  unused_qualifications
)]

//! [r2d2-cypher](https://github.com/flosse/r2d2-cypher) is a
//! [r2d2](https://github.com/sfackler/r2d2) connection pool for
//! [rusted-cypher](https://github.com/livioribeiro/rusted-cypher).
//!
//! [![](http://meritbadge.herokuapp.com/r2d2_cypher)](https://crates.io/crates/r2d2_cypher)
//! [![Build Status](https://travis-ci.org/flosse/r2d2-cypher.svg?branch=master)](https://travis-ci.org/flosse/r2d2-cypher)
//!
//! # Example
//!
//! ```
//! extern crate r2d2;
//! extern crate r2d2_cypher;
//!
//! use r2d2::Pool;
//! use r2d2_cypher::CypherConnectionManager;
//!
//! pub fn main() {
//!   let db_url  = "http://neo4j:neo4j@127.0.0.1:7474/db/data";
//!   let manager = CypherConnectionManager{url:db_url.to_owned()};
//!   let pool    = Pool::builder().max_size(5).build(manager).unwrap();
//!   let client  = pool.clone().get().unwrap();
//!   let result  = client.cypher().exec("MATCH (n)-[r]->() RETURN n");
//! }
//! ```
use futures::prelude::*;
use futures01::Future;
use rusted_cypher::error::GraphError;
use rusted_cypher::GraphClient;

/// A struct that holds connection specific information.
#[derive(Debug)]
pub struct CypherConnectionManager {
  /// the URL to the database
  pub url: String,
}

#[cfg(not(any(feature = "bb8", feature = "l337")))]
compile_error!("Either feature \"bb8\" or \"l337\" must be enabled for this crate.");

#[cfg(feature = "bb8")]
impl bb8::ManageConnection for CypherConnectionManager {
  type Connection = GraphClient;
  type Error = GraphError;

  fn connect(&self) -> Box<dyn Future<Item = Self::Connection, Error = Self::Error> + Send> {
    Box::new(GraphClient::connect(self.url.to_owned()).boxed().compat())
  }

  fn is_valid(
    &self,
    conn: Self::Connection,
  ) -> Box<dyn Future<Item = Self::Connection, Error = (Self::Error, Self::Connection)> + Send> {
    Box::new(
      async {
        let res = conn.exec("RETURN 1").await;

        match res {
          Ok(_) => Ok(conn),
          Err(err) => Err((err, conn)),
        }
      }
        .boxed()
        .compat(),
    )
  }

  fn has_broken(&self, _: &mut Self::Connection) -> bool {
    false
  }
}

#[cfg(feature = "l337")]
impl l337::ManageConnection for CypherConnectionManager {
  type Connection = GraphClient;
  type Error = GraphError;

  fn connect(
    &self,
  ) -> Box<dyn Future<Item = Self::Connection, Error = l337::Error<Self::Error>> + Send> {
    Box::new(
      GraphClient::connect(self.url.to_owned())
        .map_err(|e| l337::Error::External(e))
        .boxed()
        .compat(),
    )
  }

  fn is_valid(
    &self,
    conn: Self::Connection,
  ) -> Box<dyn Future<Item = (), Error = l337::Error<Self::Error>>> {
    Box::new(
      async move {
        let res = conn.exec("RETURN 1").await;

        match res {
          Ok(_) => Ok(()),
          Err(err) => Err(l337::Error::External(err)),
        }
      }
        .boxed()
        .compat(),
    )
  }

  fn has_broken(&self, _: &mut Self::Connection) -> bool {
    false
  }

  fn timed_out(&self) -> l337::Error<Self::Error> {
    l337::Error::External(GraphError::Other("Timed out".to_owned()))
  }
}
