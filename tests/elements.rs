#[poly_enum::poly_enum]
#[repr(u32)]
enum Elements {
	#[poly_enum(NonMetal)]
	Carbon,
	#[poly_enum(Oxidizer, NonMetal)]
	Florine,
	#[poly_enum(Metal)]
	Iron,
	#[poly_enum(Alkali, Metal)]
	Lithium,
	#[poly_enum(Oxidizer, NonMetal)]
	Oxygen,
	#[poly_enum(Alkali, Metal)]
	Sodium,
}

#[test]
fn cast_carbon() {
	use poly_enum::Cast;
	let e_carbon = Elements::Carbon;
	let nm_carbon: NonMetal = e_carbon.cast().unwrap();
	let ox_carbon: Option<Oxidizer> = nm_carbon.cast();
	assert!(ox_carbon.is_none());
}

#[test]
fn cast_florine() {
	use poly_enum::Cast;
	assert!(Cast::<Metal>::cast(Oxidizer::Florine).is_none());
	assert!(Cast::<NonMetal>::cast(Oxidizer::Florine).is_some());
	assert!(Cast::<Alkali>::cast(Oxidizer::Florine).is_none());
}

#[test]
fn cast_iron() {
	use poly_enum::Cast;
	assert!(Cast::<Alkali>::cast(Elements::Iron).is_none());
	assert!(Cast::<Metal>::cast(Elements::Iron).is_some());
	assert!(Cast::<NonMetal>::cast(Elements::Iron).is_none());
	assert!(Cast::<Oxidizer>::cast(Elements::Iron).is_none());
}

#[test]
fn cast_by_ref() {
	use poly_enum::CastRef;
	assert!(CastRef::<Metal>::cast_ref(&Elements::Florine).is_none());
}
