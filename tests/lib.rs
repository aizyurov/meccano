extern crate meccano;

use meccano::{Builder,Context};
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
	let mut builder = Builder::new();
	builder.add(|ctx: &Context| Dependent{dependency: ctx.get::<Arc<Dependency>>()});
	builder.add(|ctx: &Context| Arc::new(Dependency{value: ctx.named::<i32>("value")}));
	builder.label("value", |ctx: &Context| 365);
	let ctx = builder.build();
	let val = ctx.named::<i32>("value");
	assert_eq!(val, 365);	 		
	let dependent = ctx.get::<Dependent>();
	assert_eq!(dependent.dependency.value, 365);	 		
}

//#[test]
//fn init_is_lazy() {
//	let mut builder = Builder::new();
//	builder.add(|ctx: &Context| Dependent{dependency: ctx.get::<Arc<Dependency>>()});
//	builder.add(|ctx: &Context| Arc::new(Dependency{value: ctx.named::<i32>("value")}));
//	builder.label("other_value", |ctx: &Context| 365);
//	// should not fail because nothing is got from context
//	let ctx = builder.build();
//}
//
//#[test]
//#[should_panic(expected = "Unresolved dependency label")]
//fn unresoved_label() {
//	let mut builder = Builder::new();
//	builder.add(|ctx: &Context| Dependent{dependency: ctx.get::<Arc<Dependency>>()});
//	builder.add(|ctx: &Context| Arc::new(Dependency{value: ctx.named::<i32>("value")}));
//	builder.label("other_value", |ctx: &Context| 365);
//	let ctx = builder.build();
//	let dependent = ctx.get::<Dependent>();
//}
//
//#[test]
//#[should_panic(expected = "Unresolved dependency type")]
//fn unresoved_type() {
//	let mut builder = Builder::new();
//	builder.add(|ctx: &Context| Dependent{dependency: ctx.get::<Arc<Dependency>>()});
//	builder.add(|ctx: &Context| Arc::new(Mutex::new(Dependency{value: ctx.named::<i32>("value")})));
//	builder.label("value", |ctx: &Context| 365);
//	let ctx = builder.build();
//	let dependent = ctx.get::<Dependent>();
//}
//
//struct Short {
//	next: Arc<Short>,
//}
//
//#[test]
//#[should_panic(expected = "already borrowed")]
//fn short_circuit() {
//	let mut builder = Builder::new();
//	builder.add(|ctx: &Context| Arc::new(Short{next: ctx.get::<Arc<Short>>()}));
//	let ctx = builder.build();
//	let short = ctx.get::<Arc<Short>>();
//}
//
//struct Head {
//	tail: Arc<Tail>
//}
//
//struct Tail {
//	head: Arc<Head>
//}
//
//#[test]
//#[should_panic(expected = "already borrowed")]
//fn cycle() {
//	let mut builder = Builder::new();
//	builder.add(|ctx: &Context| Arc::new(Head{tail: ctx.get::<Arc<Tail>>()}));
//	builder.add(|ctx: &Context| Arc::new(Tail{head: ctx.get::<Arc<Head>>()}));
//	let head = builder.build().get::<Arc<Head>>();
//}
//
//#[test]
//fn has_contains() {
//	let mut builder = Builder::new();
//	builder.add(|ctx: &Context| Dependent{dependency: ctx.get::<Arc<Dependency>>()});
//	builder.add(|ctx: &Context| Arc::new(Dependency{value: ctx.named::<i32>("value")}));
//	builder.label("value", |ctx: &Context| 365);
//	let ctx = builder.build();
//	assert!(ctx.has::<Arc<Dependency>>());
//	assert!(ctx.has::<Dependent>());
//	assert!(!ctx.has::<Arc<Dependent>>());
//	assert!(ctx.contains::<i32>("value"));
//	assert!(!ctx.contains::<i32>("other_value"));
//	assert!(!ctx.contains::<i64>("value"));
//}
//
//#[test]
//fn send() {
//	let mut builder = Builder::new();
//	builder.add(|ctx: &Context| Dependent{dependency: ctx.get::<Arc<Dependency>>()});
//	builder.add(|ctx: &Context| Arc::new(Dependency{value: ctx.named::<i32>("value")}));
//	builder.label("value", |ctx: &Context| 365);
//	let ctx = builder.build();
//	let j = std::thread::spawn(move || ctx.get::<Dependent>());
//	j.join();
//	
//}