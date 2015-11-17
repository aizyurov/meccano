extern crate meccano;

use meccano::{Rules, Context};
use std::sync::{Arc, Mutex};

struct ConnectionPool {
	size: i32,
	// other fields omitted
}

impl ConnectionPool {
	pub fn new(size: i32) -> ConnectionPool {
		ConnectionPool{size: size}
	}
}

struct Service1 {
	pool: Arc<ConnectionPool>,
}

struct Service2 {
	pool: Arc<ConnectionPool>,
}

fn main() {
	let mut rules = Rules::new();
	rules.add("", |ctx: &Context| Arc::new(Service1{pool: ctx.get::<Arc<ConnectionPool>>("")}));
	rules.add("", |ctx: &Context| Arc::new(Service2{pool: ctx.get::<Arc<ConnectionPool>>("")}));
	rules.add("", |ctx: &Context| Arc::new(ConnectionPool{size: ctx.get::<i32>("connection.pool.size")}));
	rules.add("connection.pool.size", |ctx: &Context| 32);
	let ctx = rules.commit();
	let service1 = ctx.get::<Arc<Service1>>("");
	let lock1 = std::thread::spawn(move || {
			println!("Service1 running with pool size {}", service1.pool.size);
		}
	);
	let service2 = ctx.get::<Arc<Service2>>("");
	let lock2 = std::thread::spawn(move || {
			println!("Service2 running with pool size {}", service2.pool.size);
		}
	);
	lock1.join().unwrap();
	lock2.join().unwrap();
}
