//!
//! An example of registering multiple services implementing common trait
//! and starting all services in the context
//!

extern crate meccano;

use meccano::{Rules, Context};
use std::sync::{Arc, Mutex};
use connection_pool::ConnectionPool;



pub mod connection_pool {
	use meccano::{Rules, Context};
	use std::sync::{Arc, Mutex};
	
	pub struct ConnectionPool {
		pub size: i32,
		// other fields omitted
	}

	impl ConnectionPool {
		pub fn new(size: i32) -> ConnectionPool {
			ConnectionPool{size: size}
		}
	}
	
	pub fn configure(rules: &mut Rules) {
		rules.add("defaultPool", |ctx: &Context| Arc::new(ConnectionPool{size: ctx.get::<i32>("connection.pool.size")}));
		// add default value for pool size
		set_pool_size(4, rules);
	}
	
	pub fn set_pool_size(size: i32, rules: &mut Rules) {
		rules.add("connection.pool.size", move |ctx: &Context| size);
	}
	
	pub fn inject(ctx: &Context) -> Arc<ConnectionPool> {
		ctx.get::<Arc<ConnectionPool>>("defaultPool")
	}
}

trait Service: Sync + Send {
	fn start(&self);
}

struct Service1 {
	pool: Arc<ConnectionPool>,
}

struct Service2 {
	pool: Arc<ConnectionPool>,
}

impl Service for Service1 {
	fn start(&self) {
		println!("Service1 running with pool size {}", self.pool.size);
	}
}

impl Service for Service2 {
	fn start(&self) {
		println!("Service2 running with pool size {}", self.pool.size);
	}
}


fn main() {
	let mut rules = Rules::new();
	rules.add("one", |ctx: &Context| Arc::new(Service1{pool: connection_pool::inject(ctx)}) as Arc<Service>);
	connection_pool::configure(& mut rules);
	connection_pool::set_pool_size(32, & mut rules);
	rules.add("two", |ctx: &Context| Arc::new(Service2{pool: connection_pool::inject(ctx)})  as Arc<Service>);
	let ctx = rules.commit();
	let mut services: Vec<Arc<Service>> = Vec::new();
	for name in ctx.keys::<Arc<Service>>() {
		services.push(ctx.get::<Arc<Service>>(name));
	}
	let mut locks: Vec<std::thread::JoinHandle<()>> = Vec::new();
	for service in services {
		let lock = std::thread::spawn(move || {
			service.start(); 
		});
		locks.push(lock);
	}
	for lock in locks {
		lock.join().unwrap();
	} 
}
