use proc_macro2::TokenTree;
use syn::{punctuated::Punctuated, GenericArgument, GenericParam, Ident, MetaList, PathArguments, PathSegment, ReturnType, Type};

pub fn find_generic_candidates(
	ty: &Type,
	filter: impl Fn(&Ident)->bool + Clone,
) -> Vec<Ident> {
	match ty {
		Type::Array(arr) => find_generic_candidates(&arr.elem, filter),
		Type::BareFn(bare) => {
			bare.inputs.iter().flat_map(|input| {
				find_generic_candidates(&input.ty, filter.clone())
			}).chain(
				bare.lifetimes.iter().flat_map(|b| &b.lifetimes).filter_map(|g| match g {
					GenericParam::Const(c) => filter(&c.ident).then_some(&c.ident).cloned(),
					GenericParam::Lifetime(lt) => filter(&lt.lifetime.ident).then_some(&lt.lifetime.ident).cloned(),
					GenericParam::Type(ty) => filter(&ty.ident).then_some(&ty.ident).cloned(),
				})
			).chain(match &bare.output {
				ReturnType::Default => Default::default(),
				ReturnType::Type(_, ty) => find_generic_candidates(ty, filter.clone())
			}).collect()
		},
		Type::Group(group) => find_generic_candidates(&group.elem, filter),
		Type::Paren(paren) => find_generic_candidates(&paren.elem, filter),
		Type::Path(path) => {
			if let Some(qs) = &path.qself {
				find_generic_candidates(&qs.ty, filter.clone()).into_iter().chain(
					find_generic_candidates_path_segments(&path.path.segments, filter.clone())
				).collect()
			} else {
				path.path.segments.first().iter().filter_map(|segment| {
					filter(&segment.ident).then_some(&segment.ident).cloned()
				}).chain(find_generic_candidates_path_segments(&path.path.segments, filter.clone())).collect()
			}
		}
		Type::Ptr(ptr) => find_generic_candidates(&ptr.elem, filter),
		Type::Reference(r) => find_generic_candidates(&r.elem, filter.clone()).into_iter().chain(
			r.lifetime.iter().filter_map(|lt| {
				filter(&lt.ident).then_some(&lt.ident).cloned()
			})
		).collect(),
		Type::Slice(s) => find_generic_candidates(&s.elem, filter),
		_ => Default::default(),
	}
}

pub fn find_generic_candidates_path_segments<P>(
	segments: &Punctuated<PathSegment, P>,
	filter: impl Fn(&Ident)->bool + Clone
) -> Vec<Ident> {
	segments.iter().flat_map(|segment| match &segment.arguments {
		PathArguments::AngleBracketed(angled) => {
			angled.args.iter().flat_map(|a| match a {
				GenericArgument::Lifetime(lt) => filter(&lt.ident).then_some(&lt.ident).cloned().into_iter().collect(),
				GenericArgument::Type(ty) => find_generic_candidates(ty, filter.clone()),
				_ => vec![],
			}).collect()
		},
		PathArguments::None => vec![],
		PathArguments::Parenthesized(parens) => {
			parens.inputs.iter().flat_map(|ty| find_generic_candidates(ty, filter.clone())).chain(match &parens.output {
				ReturnType::Default => Default::default(),
				ReturnType::Type(_, ty) => find_generic_candidates(ty, filter.clone())
			}).collect()
		},
	}).collect()
}

pub fn parse_attr_variants(list: &MetaList) -> syn::Result<Vec<Ident>> {
	let mut idents = Vec::new();
	let mut expecting_comma = false;
	for e in list.tokens.clone().into_iter() {
		match e {
			TokenTree::Ident(ident) if !expecting_comma => idents.push(ident),
			TokenTree::Punct(p) if expecting_comma && p.as_char() == ',' => {},
			_ if expecting_comma => Err(syn::Error::new(e.span(), "Expected ','"))?,
			_ => Err(syn::Error::new(e.span(), "Expected identifier"))?,
		}
		expecting_comma = !expecting_comma;
	}
	Ok(idents)
}
