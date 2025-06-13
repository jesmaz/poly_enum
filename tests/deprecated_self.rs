#![allow(deprecated)]

use std::{rc::Rc, sync::Arc};

use poly_enum::PolyEnum;

#[derive(Clone, PolyEnum)]
#[poly_derive(Clone)]
enum AnyPtr {
	#[poly_enum(RcPtr)]
	Arc(Arc<Self>),
	#[poly_enum(BoxPtr)]
	Box(Box<Self>),
	#[poly_enum(RcPtr)]
	Rc(Rc<Self>),
	#[poly_enum(NotAPointer)]
	None,
}
