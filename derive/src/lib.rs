mod derive;
mod attr_proc;
mod util;

#[proc_macro_attribute]
pub fn poly_enum(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
	attr_proc::poly_enum(attr, item)
}

#[proc_macro_derive(PolyEnum, attributes(poly_derive, poly_enum))]
pub fn poly_enum_derive(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
	derive::poly_enum_derive(item)
}
