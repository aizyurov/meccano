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
	service1: Arc<Service1>,
}

fn main() {
	let mut rules = Rules::new();
	rules.add("", |ctx: &Context| Arc::new(Service1{pool: ctx.get::<Arc<ConnectionPool>>("")}));
	rules.add("", |ctx: &Context| Arc::new(Service2{pool: ctx.get::<Arc<ConnectionPool>>(""), service1: ctx.get::<Arc<Service1>>("")}));
	rules.add("", |ctx: &Context| Arc::new(ConnectionPool{size: ctx.get::<i32>("connection.pool.size")}));
	rules.add("connection.pool.size", |ctx: &Context| 32);
	let ctx = Arc::new(Mutex::new(rules.commit()));
	let ctx1 = ctx.clone();
	let lock1 = std::thread::spawn(move || {
			let service = ctx1.lock().unwrap().get::<Arc<Service1>>(""); 
			println!("Service1 running with pool size {}", service.pool.size);
		}
	);
	let lock2 = std::thread::spawn(move || {
			let service = ctx.lock().unwrap().get::<Arc<Service2>>(""); 
			println!("Service2 running with pool size {}", service.pool.size);
		}
	);
	lock1.join().unwrap();
	lock2.join().unwrap();
}
