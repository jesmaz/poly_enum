use std::{rc::Rc, sync::Arc};

use poly_enum::PolyEnum;

#[derive(PolyEnum)]
enum AnyPtr<T> {
	#[poly_enum(RcPtr)]
	Arc(Arc<T>),
	_Box(Box<T>),
	#[poly_enum(RcPtr)]
	Rc(Rc<T>),
	#[poly_enum(NotAPointer)]
	None,
}
