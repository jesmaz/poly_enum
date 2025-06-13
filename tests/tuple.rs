#[poly_enum::poly_enum]
#[repr(u32)]
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
	#[poly_enum(Pair)]
	V2(u64, u64),
}
