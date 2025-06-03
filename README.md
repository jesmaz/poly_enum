Hierarchical polymorphism with enums

This crate provides an easy way to create complex polymorphic hierarchies powered by enums.

# Examples
```
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

impl Alkali {
    fn explodes_in_water(&self) -> bool {true}
}

let metal = Metal::Sodium;
let alkali: Alkali = metal.cast().unwrap();
assert!(alkali.explodes_in_water());
```
This will generate 4 enums in addition to the base Elements enum: NonMetal, Oxidizer, Metal, and Alkali.
```
enum Alkali {
    Sodium,
    Lithium,
}

enum Metal {
    Iron,
    Lithium,
    Sodium,
}

enum NonMetal {
    Carbon,
    Oxygen,
    Florine,
}

enum Oxidizer {
    Oxygen,
    Florine,
}
```
In addition it will implement PolyEnum to allow casting between these enums.

# Limitations
- No dynamic dispatch other than the match expression
- Separate hierarchies are not cross compatible
