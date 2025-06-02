use std::{rc::Rc, sync::Arc};

use poly_enum::PolyEnum;

#[derive(PolyEnum)]
enum AnyPtr<T> where T: Clone {
	#[poly_enum(RcPtr)]
	Arc(Arc<T>),
	#[poly_enum(BoxPtr)]
	Box(Box<T>),
	#[poly_enum(RcPtr)]
	Rc(Rc<T>),
	#[poly_enum(NotAPointer)]
	None,
}
