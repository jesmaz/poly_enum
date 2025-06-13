use std::{rc::Rc, sync::Arc};

#[poly_enum::poly_enum]
#[repr(u32)]
enum AnyPtr<T> {
	#[poly_enum(RcPtr)]
	Arc(Arc<T>),
	_Box(Box<T>),
	#[poly_enum(RcPtr)]
	Rc(Rc<T>),
	#[poly_enum(NotAPointer)]
	None,
}
