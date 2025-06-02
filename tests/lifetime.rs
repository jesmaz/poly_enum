use poly_enum::PolyEnum;

#[derive(PolyEnum)]
enum Value<'a> {
	#[poly_enum(Ref)]
	Str(&'a str),
	#[poly_enum(Owned)]
	String(String),
}
