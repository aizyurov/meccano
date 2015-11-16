//! A simple dependency injection framework
//!

use std::collections::HashMap;
use std::cell::RefCell;
use std::any::Any;
// use std::cell::BorrowState;


extern crate anymap;

use anymap::AnyMap;

pub struct Context {
	map: AnyMap,
}

pub trait Constructor<T: Any> : Any {
	fn construct(& self, & Context) -> T; 
}

impl<F, T> Constructor<T> for F where F: Fn(& Context) -> T, F: Any + Sync + Send, T: Any 
    {
    fn  construct(& self, ctx: & Context) -> T {
        (*self)(ctx)
    }
}
    
enum CachedObject<T> {
	New,
	UnderConstruction,
	Ready(T)	
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

	pub fn get<T: Any + Clone>(&self) -> T {
		self.named::<T>("")
	}

	pub fn named<T: Any + Clone>(&self, name: &str) -> T {
		let inner_map = self.map.get::<HashMap<&'static str, RefCell<Binding<T>>>>().expect("Unresolved dependency type");
		let binding_ref = inner_map.get(name).expect(&format!("Unresolved dependency label {}", &name));
		// TODO when borrow_state is available, check it and panic with "cyclic deoendency"
		binding_ref.borrow_mut().get(self)
	}
	
	pub fn has<T: Any + Clone>(&self) -> bool {
		self.contains::<T>("")
	}
	
	pub fn contains<T: Any + Clone>(&self, name: &str) -> bool {
		match self.map.get::<HashMap<&'static str, RefCell<Binding<T>>>>() {
			None => false,
			Some(x) => match x.get(name) {
				None => false,
				Some(_) => true
			}
		} 
	}
}

pub struct Builder {
	map: AnyMap,	
}

impl Builder {

	pub fn new() -> Builder {
		Builder{map: AnyMap::new()}
	}

	pub fn label<T: Any + Clone, C>(&mut self, name: &'static str, ctr: C)
		where C: Constructor<T>		
	{
		if !self.map.contains::<HashMap<&'static str, RefCell<Binding<T>>>>() {
			self.map.insert(HashMap::<&'static str, RefCell<Binding<T>>>::new());
		}
		let inner_map =  self.map.get_mut::<HashMap<&'static str, RefCell<Binding<T>>>>().expect("surprise!");
		inner_map.insert(name, RefCell::new(Binding::new(Box::new(ctr))));
	}

	pub fn add<T: Any + Clone, C>(&mut self, ctr: C)
		where C: Constructor<T>		
	{
		self.label("", ctr);
	}
	
	pub fn build(self) -> Context {
		Context{map: self.map}
	} 
	
}



#[test]
fn it_works() {
	println!("test started");
	let mut builder = Builder::new();
	builder.add(|ctx: & Context| {32});
	builder.add(|ctx: & Context| {33});
	let context = builder.build();
	assert_eq!(context.get::<i32>(), 33);
}


