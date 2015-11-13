//! A simple dependency injection framework
//!

use std::collections::HashMap;
use std::cell::RefCell;
use std::any::Any;

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
	obj: CachedObject<T>,
}

impl <T: Any + Clone> Binding<T> {

	fn new(constructor: Box<Constructor<T>>) -> Binding<T> {
		Binding{ctr: constructor, obj: CachedObject::New}
	}
	
}

impl Context {

	pub fn get<T: Any + Clone>(&self) -> T {
		self.named::<T>("")
	}

	pub fn named<T: Any + Clone>(&self, name: &str) -> T {
		let inner_map = self.map.get::<HashMap<String, RefCell<Binding<T>>>>().expect("Unresolved context object");
		let binding_ref = inner_map.get(name).expect("Unresolved context object");
		let cached: Option<T>;
		{
			let binding = binding_ref.borrow();
			cached = match binding.obj {
				CachedObject::New => Option::None,
				CachedObject::Ready(ref ready) => Option::Some(ready.clone()), 
				CachedObject::UnderConstruction => panic!("Cyclic dependency in context"),
			}
		}
		// borrow ends here
		match cached {
			Some(x) => x,
			None => {
				// it is safe to borrow mutable reference
				let mut binding = binding_ref.borrow_mut();
				// protect fron duplicate borrow_nut
				binding.obj = CachedObject::UnderConstruction;
				let t = binding.ctr.construct(self); 
				binding.obj = CachedObject::Ready(t.clone());
				t
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

	pub fn label<T: Any + Clone + Send, C>(&mut self, name: &str, ctr: C)
		where C: Constructor<T>		
	{
		if !self.map.contains::<HashMap<String, RefCell<Binding<T>>>>() {
			self.map.insert(HashMap::<String, RefCell<Binding<T>>>::new());
		}
		let inner_map =  self.map.get_mut::<HashMap<String, RefCell<Binding<T>>>>().expect("surprise!");
		inner_map.insert(name.to_string(), RefCell::new(Binding::new(Box::new(ctr))));
	}

	pub fn add<T: Any + Clone + Sync + Send, C>(&mut self, ctr: C)
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
	println!("got from context {}", context.get::<i32>());
}


