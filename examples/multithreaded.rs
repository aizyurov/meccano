//!
//! Similar to iterate.rs but demonstrates instantiation of services
//! in different threads
//!

extern crate meccano;

use meccano::{Rules, Context};
use std::sync::{Arc, Mutex};


pub struct ConnectionPool {
	size: i32,
	// other fields omitted
}

impl ConnectionPool {
	pub fn new(size: i32) -> ConnectionPool {
		ConnectionPool{size: size}
	}
}

pub mod pool_config {
	
	pub use ConnectionPool;
	use meccano::{Rules, Context};
	use std::sync::{Arc, Mutex};
	
	pub fn configure(rules: &mut Rules) {
		rules.add("defaultPool", |ctx: &Context| Arc::new(ConnectionPool{size: ctx.get::<i32>("connection.pool.size")}));
		// add default value for pool size
		set_pool_size(4, rules);
	}
	
	pub fn set_pool_size(size: i32, rules: &mut Rules) {
		rules.add("connection.pool.size", move |ctx: &Context| size);
	}
	
	pub fn pool(ctx: &Context) -> Arc<ConnectionPool> {
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
	rules.add("one", |ctx: &Context| Arc::new(Service1{pool: pool_config::pool(ctx)}) as Arc<Service>);
	pool_config::configure(& mut rules);
	pool_config::set_pool_size(32, & mut rules);
	rules.add("two", |ctx: &Context| Arc::new(Service2{pool: pool_config::pool(ctx)})  as Arc<Service>);
	let ctx = Arc::new(Mutex::new(rules.commit()));
	let mut locks: Vec<std::thread::JoinHandle<()>> = Vec::new();
	let mut services: Vec<Arc<Service>> = Vec::new();
	{
		let locked_context = ctx.lock().unwrap();
		let names = locked_context.keys::<Arc<Service>>().collect::<Vec<&str>>();
		for name in names {
			println!("Getting {}", name);
			services.push(locked_context.get::<Arc<Service>>(name));
			println!("Got {}", name);
		}
	}
	println!("Services collected");
	for service in services {
		let ctx = ctx.clone();
		let lock = std::thread::spawn(move || {
			service.start(); 
		});
		locks.push(lock);
	}
	for lock in locks {
		lock.join().unwrap();
	} 
}
