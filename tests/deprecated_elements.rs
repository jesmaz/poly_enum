#![allow(deprecated)]

use poly_enum::PolyEnum;

#[derive(PolyEnum)]
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
	let e_carbon = Elements::Carbon;
	let nm_carbon: NonMetal = e_carbon.cast().unwrap();
	let ox_carbon: Option<Oxidizer> = nm_carbon.cast();
	assert!(ox_carbon.is_none());
}

#[test]
fn cast_florine() {
	assert!(PolyEnum::<Metal>::cast(Oxidizer::Florine).is_none());
	assert!(PolyEnum::<NonMetal>::cast(Oxidizer::Florine).is_some());
	assert!(PolyEnum::<Alkali>::cast(Oxidizer::Florine).is_none());
}

#[test]
fn cast_iron() {
	assert!(PolyEnum::<Alkali>::cast(Elements::Iron).is_none());
	assert!(PolyEnum::<Metal>::cast(Elements::Iron).is_some());
	assert!(PolyEnum::<NonMetal>::cast(Elements::Iron).is_none());
	assert!(PolyEnum::<Oxidizer>::cast(Elements::Iron).is_none());
}
