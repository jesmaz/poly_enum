#![allow(deprecated)]

use poly_enum::PolyEnum;

#[derive(PolyEnum)]
enum Value {
	#[poly_enum(Half, Float)]
	F32{v: f32},
	#[poly_enum(Float)]
	F64{v: f64},
	#[poly_enum(Half, Int)]
	I32{v: i32},
	#[poly_enum(Int)]
	I64{v: i64},
	#[poly_enum(Int)]
	U64{v: u64},
	#[poly_enum(Pair)]
	V2{v: u64, u: u64},
}
