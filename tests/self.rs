use std::{rc::Rc, sync::Arc};

#[poly_enum::poly_enum]
#[repr(u32)]
#[derive(Clone)]
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
