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
	
}

impl Context {

	pub fn get<T: Any + Clone>(&self) -> T {
		self.named::<T>("")
	}

	pub fn named<T: Any + Clone>(&self, name: &str) -> T {
		let inner_map = self.map.get::<HashMap<String, RefCell<Binding<T>>>>().expect("Unresolved dependency type");
		let binding_ref = inner_map.get(name).expect(&format!("Unresolved dependency label {}", &name));
		let cached_value: Option<T>;
		// This staff waits for 1.5. For the time being we are getting "Already borrowed" instead of
		// "Cyclic dependency"
//		match binding_ref.borrow_state() {
//			BorrowState::Unused => {
//				let mut binding = binding_ref.borrow();
//				match binding.obj {
//					Option::Some(ref value) => cached_value = Option::Some(value.clone()),
//					Option::None => { cached_value = Option::None }
//				}
//			}
//			// if already borrowed, the same object is requested while it is under constuction
//			_ => panic!("Cyclic dependency")
//		}

		{
			let binding = binding_ref.borrow();
			match binding.obj {
				Some(ref value) => cached_value = Option::Some(value.clone()),
				None =>  cached_value = Option::None
			}
		}
		match cached_value {
			Some(value) => value,
			None => {	
				let mut binding = binding_ref.borrow_mut();
				let t = binding.ctr.construct(self);
				binding.obj = Some(t.clone());
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

	pub fn label<T: Any + Clone, C>(&mut self, name: &str, ctr: C)
		where C: Constructor<T>		
	{
		if !self.map.contains::<HashMap<String, RefCell<Binding<T>>>>() {
			self.map.insert(HashMap::<String, RefCell<Binding<T>>>::new());
		}
		let inner_map =  self.map.get_mut::<HashMap<String, RefCell<Binding<T>>>>().expect("surprise!");
		inner_map.insert(name.to_string(), RefCell::new(Binding::new(Box::new(ctr))));
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


