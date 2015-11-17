//! # Meccano
//! A simple dependency injection framework
//!
//! # Design
//! The central structure is [Context](struct.Context.html). Objects in the context can be accessed by type and name pair (names must be
//! distinct within one type only).
//!
//! Context is lazy. It creates an object on demand; next calls return a copy. Typically you want to share
//! the same object; in this case context holds Arc<Something> or Arc<Mutex<Something>> - a copy of Arc is
//! exactly you need for sharing.
//! If a [Constructor](trait.Constructor.html) of an object requires non-existing reference to another object or 
//! cyclic depencency is discovered, the Context panics. Typically you cannot do anything if context is misconfigured.
//!
//! To construct the Context, first fill [Rules](struct.Rules.html) - they define how to construct an object of given type and with given name.
//! # Examples
//!
//! ## Basic
//! ```
//! extern crate meccano;
//! use meccano::{Rules, Context};
//! use std::sync::{Arc, Mutex};
//! 
//! struct ConnectionPool {
//! 	size: i32,
//! 	// other fields omitted
//! }
//! impl ConnectionPool {
//! 	pub fn new(size: i32) -> ConnectionPool {
//! 		ConnectionPool{size: size}
//! 	}
//! }
//! 
//! struct Service {
//!     // there will be other services sharing the same ConnectionPool
//! 	pool: Arc<ConnectionPool>,
//! }
//! 
//! fn main() {
//! 	let mut rules = Rules::new();
//!
//!     // Rule for construction of Service takes Arc<ConnectionPool> from context
//!
//! 	rules.add("", |ctx: &Context| Arc::new(Service{pool: ctx.get::<Arc<ConnectionPool>>("")}));
//!     // Construction of ConnectionPool in turn takes pool size from context
//! 	rules.add("", |ctx: &Context| Arc::new(ConnectionPool{size: ctx.get::<i32>("connection.pool.size")}));
//!     // in real  life you will take it from config file, command line args, ...
//! 	rules.add("connection.pool.size", |ctx: &Context| 10);
//! 	let ctx = rules.commit();
//! 	let service = ctx.get::<Arc<Service>>("");
//! 	let lock = std::thread::spawn(move || {
//! 			println!("Service running with pool size {}", service.pool.size);
//! 		}
//! 	);
//! 	lock.join().unwrap();
//! }
//! ```


use std::collections::BTreeMap;
use std::cell::RefCell;
// use std::cell::BorrowState;
use std::any::TypeId;


extern crate anymap;

use anymap::Map;
use anymap::any::Any as AnymapAny;
use std::any::Any;
use std::iter::{Iterator, Empty};

/// Can construct objects of given type using other objects from Context
/// It is implemented by functions Fn(& Context) -> T, you can use closures wherever Constructor is needed
pub trait Constructor<T: Any> : Any + Send {
	/// Construct new object using context
	fn construct(& self, & Context) -> T; 
}

impl<F, T> Constructor<T> for F where F: Fn(& Context) -> T, F: Any + Send, T: Any {
    fn  construct(& self, ctx: & Context) -> T {
        (*self)(ctx)
    }
}

/// Holds rules for object construction and cached instances. 
pub struct Context {
	map: Map<AnymapAny + Send>,
}

    
struct Binding<T> {
	ctr: Box<Constructor<T>>,
	obj: Option<T>,
}

impl <T: Any + Clone> Binding<T> {

	fn new(constructor: Box<Constructor<T>>) -> Binding<T> {
		Binding{ctr: constructor, obj: Option::None}
	}
	
	fn get(&mut self, ctx: &Context) -> T {
		if let Some(ref x) = self.obj {
			return x.clone();
		} 
		let value = self.ctr.construct(ctx);
		self.obj = Some(value.clone());
		value
	}
	
}

impl Context {

/// Get an object with given type and name. If the object does not exist yet, it is created,
/// otherwise a clone is returned.
/// # Panics
/// Unresolved dependency or cyclic dependency
	pub fn get<T: Any + Clone + Send>(&self, name: &str) -> T {
		let inner_map = self.map.get::<BTreeMap<&'static str, RefCell<Binding<T>>>>().expect("Unresolved dependency type");
		let binding_ref = inner_map.get(name).expect(&format!("Unresolved dependency label {}", &name));
		// TODO when borrow_state is available, check it and panic with "cyclic deoendency"
		binding_ref.borrow_mut().get(self)
	}

/// All avaliable in the Context names for given type. Due to Context lazyness it is not guaranteed that
/// subsequent get() for given name succeeds.	
	pub fn keys<'a, T: Any + Clone + Send>(&'a self) -> Box<Iterator<Item=&'static str> + 'a> {
		let maybe_map = self.map.get::<BTreeMap<&'static str, RefCell<Binding<T>>>>();
		match maybe_map {
			Some(ref x) => Box::new(x.keys().map(|v| *v)),
			None => { Box::new(std::iter::empty::<&'static str>()) }
		}
	}
}

/// Definition of object construction rules for Context
pub struct Rules {
	map: Map<AnymapAny + Send>,	
}

impl Rules {

	/// Empty Rules
	pub fn new() -> Rules {
		Rules{map: Map::new()}
	}

	/// Defines Constructor for given (type, name) pair.
	/// Functions of type |ctx: &Context| -> T implement Constructor, you can use closures as argument.
	pub fn add<T: Any + Clone + Send, C>(&mut self, name: &'static str, ctr: C)
		where C: Constructor<T>		
	{
		if !self.map.contains::<BTreeMap<&'static str, RefCell<Binding<T>>>>() {
			self.map.insert(BTreeMap::<&'static str, RefCell<Binding<T>>>::new());
		}
		let inner_map =  self.map.get_mut::<BTreeMap<&'static str, RefCell<Binding<T>>>>().expect("surprise!");
		inner_map.insert(name, RefCell::new(Binding::new(Box::new(ctr))));
	}

	/// Stop editing and produce Context. Self is consumed.
	pub fn commit(self) -> Context {
		Context{map: self.map}
	} 
	
}



#[test]
fn it_works() {
	println!("test started");
	let mut rules = Rules::new();
	rules.add("", |ctx: & Context| {32});
	rules.add("", |ctx: & Context| {33});
	let context = rules.commit();
	assert_eq!(context.get::<i32>(""), 33);
}


