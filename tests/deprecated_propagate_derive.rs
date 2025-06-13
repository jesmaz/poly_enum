#![allow(deprecated)]

use poly_enum::PolyEnum;

#[derive(Clone, Copy, Debug, PolyEnum)]
#[poly_derive(Clone, Copy, Debug)]
enum Value {
	#[poly_enum(Half, Float)]
	F32(f32),
	#[poly_enum(Float)]
	F64(f64),
	#[poly_enum(Half, Int)]
	I32(i32),
	#[poly_enum(Int)]
	I64(i64),
	#[poly_enum(Int)]
	U64(u64),
	_Void,
}

#[test]
#[allow(clippy::clone_on_copy)]
fn clone() {
	let _a = Int::I32(0).clone();
}
