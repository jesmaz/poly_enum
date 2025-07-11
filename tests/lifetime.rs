#[poly_enum::poly_enum]
#[repr(u32)]
enum Value<'a> {
	#[poly_enum(Ref)]
	Str(&'a str),
	#[poly_enum(Owned)]
	String(String),
}
