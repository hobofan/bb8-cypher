// Copyright (c) 2015 - 2017 Markus Kohlhase <mail@markus-kohlhase.de>

#![deny(
  missing_docs,
  missing_debug_implementations,
  missing_copy_implementations,
  trivial_casts,
  trivial_numeric_casts,
  unsafe_code,
  unstable_features,
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

extern crate bb8;
extern crate futures;
extern crate rusted_cypher;

use futures::Future;
use rusted_cypher::error::GraphError;
use rusted_cypher::GraphClient;

/// A struct that holds connection specific information.
#[derive(Debug)]
pub struct CypherConnectionManager {
  /// the URL to the database
  pub url: String,
}

impl bb8::ManageConnection for CypherConnectionManager {
  type Connection = GraphClient;
  type Error = GraphError;

  fn connect(&self) -> Box<Future<Item = Self::Connection, Error = Self::Error> + Send> {
    Box::new(GraphClient::connect(self.url.to_owned()))
  }

  fn is_valid(
    &self,
    conn: Self::Connection,
  ) -> Box<Future<Item = Self::Connection, Error = (Self::Error, Self::Connection)> + Send> {
    Box::new(conn.exec("RETURN 1").then(|res| match res {
      Ok(_) => Ok(conn),
      Err(err) => Err((err, conn)),
    }))
  }

  fn has_broken(&self, _: &mut GraphClient) -> bool {
    false
  }
}
