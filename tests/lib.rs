extern crate meccano;

use meccano::{Rules,Context};
use std::sync::Arc;
use std::any::TypeId;
use std::sync::Mutex;

struct Dependency {
	value: i32,
}

#[derive(Clone)]
struct Dependent {
	dependency: Arc<Dependency>,
}

#[test]
fn chain() {
	let mut rules = Rules::new();
	rules.add("", |ctx: &Context| Dependent{dependency: ctx.get::<Arc<Dependency>>("")});
	rules.add("", |ctx: &Context| Arc::new(Dependency{value: ctx.get::<i32>("value")}));
	rules.add("value", |ctx: &Context| 365);
	let ctx = rules.commit();
	let val = ctx.get::<i32>("value");
	assert_eq!(val, 365);	 		
	let dependent = ctx.get::<Dependent>("");
	assert_eq!(dependent.dependency.value, 365);	 		
}

#[test]
fn init_is_lazy() {
	let mut rules = Rules::new();
	rules.add("", |ctx: &Context| Dependent{dependency: ctx.get::<Arc<Dependency>>("")});
	rules.add("", |ctx: &Context| Arc::new(Dependency{value: ctx.get::<i32>("value")}));
	rules.add("other_value", |ctx: &Context| 365);
	// should not fail because nothing is got from context
	let ctx = rules.commit();
}

#[test]
#[should_panic(expected = "Unresolved dependency label")]
fn unresoved_label() {
	let mut rules = Rules::new();
	rules.add("", |ctx: &Context| Dependent{dependency: ctx.get::<Arc<Dependency>>("")});
	rules.add("", |ctx: &Context| Arc::new(Dependency{value: ctx.get::<i32>("value")}));
	rules.add("other_value", |ctx: &Context| 365);
	let ctx = rules.commit();
	let dependent = ctx.get::<Dependent>("");
}

#[test]
#[should_panic(expected = "Unresolved dependency type")]
fn unresoved_type() {
	let mut rules = Rules::new();
	rules.add("", |ctx: &Context| Dependent{dependency: ctx.get::<Arc<Dependency>>("")});
	rules.add("", |ctx: &Context| Arc::new(Box::new(Dependency{value: ctx.get::<i32>("value")})));
	rules.add("value", |ctx: &Context| 365);
	let ctx = rules.commit();
	let dependent = ctx.get::<Dependent>("");
}

struct List {
	next: Arc<List>,
}

#[test]
#[should_panic(expected = "already borrowed")]
fn short_circuit() {
	let mut rules = Rules::new();
	rules.add("", |ctx: &Context| Arc::new(List{next: ctx.get::<Arc<List>>("")}));
	let ctx = rules.commit();
	let short = ctx.get::<Arc<List>>("");
}

struct Head {
	tail: Arc<Tail>
}

struct Tail {
	head: Arc<Head>
}

#[test]
#[should_panic(expected = "already borrowed")]
fn cycle() {
	let mut rules = Rules::new();
	rules.add("", |ctx: &Context| Arc::new(Head{tail: ctx.get::<Arc<Tail>>("")}));
	rules.add("", |ctx: &Context| Arc::new(Tail{head: ctx.get::<Arc<Head>>("")}));
	let head = rules.commit().get::<Arc<Head>>("");
}

#[test]
fn has_contains() {
	let mut rules = Rules::new();
	rules.add("", |ctx: &Context| Dependent{dependency: ctx.get::<Arc<Dependency>>("")});
	rules.add("", |ctx: &Context| Arc::new(Dependency{value: ctx.get::<i32>("value")}));
	rules.add("value", |ctx: &Context| 365);
	let ctx = rules.commit();
	assert!(ctx.contains::<Arc<Dependency>>(""));
	assert!(ctx.contains::<Dependent>(""));
	assert!(!ctx.contains::<Arc<Dependent>>(""));
	assert!(ctx.contains::<i32>("value"));
	assert!(!ctx.contains::<i32>("other_value"));
	assert!(!ctx.contains::<i64>("value"));
}
#[test]
fn send() {
	let mut rules = Rules::new();
	rules.add("", |ctx: &Context| Dependent{dependency: ctx.get::<Arc<Dependency>>("")});
	rules.add("", |ctx: &Context| Arc::new(Dependency{value: ctx.get::<i32>("value")}));
	rules.add("value", |ctx: &Context| 365);
	let ctx = rules.commit();
	let j = std::thread::spawn(move || ctx.get::<Dependent>(""));
	j.join().unwrap();
	
}

#[test]
fn multithreaded() {
	let mut rules = Rules::new();
	rules.add("", |ctx: &Context| Dependent{dependency: ctx.get::<Arc<Dependency>>("")});
	rules.add("", |ctx: &Context| Arc::new(Dependency{value: ctx.get::<i32>("value")}));
	rules.add("value", |ctx: &Context| 365);
	let ctx = rules.commit();
	let arc = Arc::new(Mutex::new(ctx));
	let mut v = Vec::new();
	for i in (1..5) {
		let arc_copy = arc.clone();
		v.push(std::thread::spawn(move || {
					let dependent = arc_copy.lock().unwrap().get::<Dependent>("");
					let res:i32 = dependent.dependency.value;
					res
					})
				);
	}
	for t in v {
		let res: i32 = t.join().unwrap();
		assert_eq!(res, 365);
	}
}

// must not compile: 
// error: the trait `core::marker::Sync` is not implemented for the type `anymap::any::Any + Send + 'static` [E0277]
//#[test]
//fn multithreaded_unsync() {
//	let mut rules = Rules::new();
//	rules.add("", |ctx: &Context| Dependent{dependency: ctx.get::<Arc<Dependency>>("")});
//	rules.add("", |ctx: &Context| Arc::new(Dependency{value: ctx.get::<i32>("value")}));
//	rules.add("value", |ctx: &Context| 365);
//	let ctx = rules.commit();
//	let arc = Arc::new(ctx);
//	let mut v = Vec::new();
//	for i in (1..5) {
//		let arc_copy = arc.clone();
//		v.push(std::thread::spawn(move || {
//					let dependent = arc_copy.get::<Dependent>("");
//					let res:i32 = dependent.dependency.value;
//					res
//					})
//				);
//	}
//	for t in v {
//		let res: i32 = t.join().unwrap();
//		assert_eq!(res, 365);
//	}
//}

#[test]
fn iterate() {
	let mut rules = Rules::new();
	rules.add("value", |ctx: &Context| 365);
	rules.add("otherValue", |ctx: &Context| 365);
	let ctx = rules.commit();
	let keys = ctx.keys::<i32>().collect::<Vec<&str>>();
	assert_eq!(keys, vec!("otherValue", "value"));
	let mut i64keys = ctx.keys::<i64>();
	assert_eq!(i64keys.next(), Option::None);
	}

trait Trait: Send + Sync {
	fn multiplicate(&self, i32) -> i32;
}

struct Impl {
	factor: i32,
}

impl Trait for Impl {
	fn multiplicate(&self, arg: i32) -> i32 {
		arg * self.factor
	}
}

#[test]
fn trait_object() {
	let mut rules = Rules::new();
	rules.add("", |ctx: &Context| Arc::new(Impl{factor: 2}) as Arc<Trait> );
	let ctx = rules.commit();
	let m = ctx.get::<Arc<Trait>>("");
	assert_eq!(m.multiplicate(100), 200);
}
