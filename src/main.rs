#![cfg_attr(feature = "serde_macros", feature(custom_derive, plugin))]
#![cfg_attr(feature = "serde_macros", plugin(serde_macros))]

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#[macro_use] extern crate clap;
#[macro_use] extern crate log;
extern crate ansi_term;
extern crate crypto;
extern crate csv;
extern crate env_logger;
extern crate hyper;
extern crate rand;
extern crate scoped_threadpool;
extern crate serde;
extern crate serde_json;

#[cfg(feature = "serde_macros")]
include!("main.rs.in");

#[cfg(not(feature = "serde_macros"))]
include!(concat!(env!("OUT_DIR"), "/main.rs"));
