use std::{rc::Rc, sync::Arc};

pub use derive::PolyEnum;
pub trait PolyEnum<T> {
	fn cast(self) -> Option<T>;
}

impl<T, U> PolyEnum<Arc<U>> for Arc<T> where T: PolyEnum<U> {
	fn cast(self) -> Option<Arc<U>> {todo!()}
}

impl<T, U> PolyEnum<Box<U>> for Box<T> where T: PolyEnum<U> {
	fn cast(self) -> Option<Box<U>> {todo!()}
}

impl<T, U> PolyEnum<Rc<U>> for Rc<T> where T: PolyEnum<U> {
	fn cast(self) -> Option<Rc<U>> {todo!()}
}

impl<T, U> PolyEnum<Vec<U>> for Vec<T> where T: PolyEnum<U> {
	fn cast(self) -> Option<Vec<U>> {todo!()}
}
