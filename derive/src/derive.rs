use std::collections::{HashMap, HashSet};

use quote::quote;
use syn::{parse, spanned::Spanned, Data, DeriveInput, Fields, GenericParam, Generics, Ident};

use crate::util::{find_generic_candidates, parse_attr_variants};

pub fn poly_enum_derive(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let derive_input = match parse::<DeriveInput>(item) {
		Ok(e) => e,
		Err(err) => {
			return err.into_compile_error().into();
		},
	};

	let enum_def = match &derive_input.data {
		Data::Enum(enum_def) => enum_def,
		_ => return quote! {compile_error!("PolyEnum can only be applied to an enum");}.into()
	};

	let mut sub_type_map: HashMap<_, HashSet<_>> = HashMap::new();
	let mut stripped_variants = Vec::with_capacity(enum_def.variants.len());
	for variant in &enum_def.variants {
		match variant.attrs.iter().filter(|attr| if let Some(ident) = attr.meta.path().get_ident() {
			ident == "poly_enum"
		} else {false}).map(|attr| {
			attr.meta.require_list().and_then(parse_attr_variants)
		}).collect::<Result<Vec<_>, _>>() {
			Ok(v) => {
				for ident in v.into_iter().flatten() {
					sub_type_map.entry(ident).or_default().insert(stripped_variants.len());
				}
			},
			Err(err) => {
				return err.into_compile_error().into();
			},
		}

		let mut stripped_variant = variant.clone();
		stripped_variant.attrs.retain(|attr| if let Some(ident) = attr.meta.path().get_ident() {
			ident != "poly_enum"
		} else {true});
		stripped_variants.push(stripped_variant);
	}

	let sub_type_map = sub_type_map.into_iter().map(|(k, variant_idx)| {
		let generics_set = derive_input.generics.params.iter().map(|p| match p {
			GenericParam::Const(c) => (c.ident.clone(), p.clone()),
			GenericParam::Lifetime(lt) => (lt.lifetime.ident.clone(), p.clone()),
			GenericParam::Type(ty) => (ty.ident.clone(), p.clone()),
		}).collect::<HashMap<_, _>>();

		let required_generics = variant_idx.iter().copied().filter_map(
			|u| stripped_variants.get(u)
		).flat_map(|v| match &v.fields {
			Fields::Named(named) => named.named.clone(),
			Fields::Unit => Default::default(),
			Fields::Unnamed(unnamed) => unnamed.unnamed.clone(),
		}).flat_map(|field| {
			find_generic_candidates(&field.ty, |ident| generics_set.contains_key(ident))
		}).collect::<HashSet<_>>();

		let mut generics = Generics::default();
		for required in &required_generics {
			if let Some(g) = generics_set.get(required) {
				generics.params.push(g.clone());
			}
			generics.where_clause = derive_input.generics.where_clause.clone();
		}
		(k, (variant_idx, generics))
	}).collect::<HashMap<_, _>>();

	let sub_types = sub_type_map.iter().map(|(k, (variant_idx, generics))| {
		let variants = variant_idx.iter().copied().filter_map(|u| stripped_variants.get(u));
		let enum_ident = &derive_input.ident;
		let from_variant = variant_idx.iter().copied().filter_map(|u| stripped_variants.get(u)).map(|v| {
			let ident = &v.ident;
			match &v.fields {
				Fields::Named(named) => {
					let idents = named.named.iter().flat_map(|f| {f.ident.as_ref()}).collect::<Vec<_>>();
					let conversions = named.named.iter().map(|f| if find_generic_candidates(&f.ty, |ident| ident == "Self").is_empty() {
						let ident = f.ident.as_ref();
						quote! {#ident}
					} else {
						let ident = f.ident.as_ref();
						quote! {#ident: #ident.cast().unwrap()}
					});
					quote! {#k::#ident{#(#idents),*} => #enum_ident::#ident{#(#conversions),*},}
				},
				Fields::Unit => quote! {#k::#ident => #enum_ident::#ident,},
				Fields::Unnamed(unnamed) => {
					let idents = unnamed.unnamed.iter().enumerate().map(|(idx, f)| {
						Ident::new(&format!("e_{idx}"), f.span())
					}).collect::<Vec<_>>();
					let conversions = unnamed.unnamed.iter().enumerate().map(|(idx, f)| {
						if find_generic_candidates(&f.ty, |ident| ident == "Self").is_empty() {
							let ident = Ident::new(&format!("e_{idx}"), f.span());
							quote! {#ident}
						} else {
							let ident = Ident::new(&format!("e_{idx}"), f.span());
							quote! {#ident.cast().unwrap()}
						}
					});
					quote! {#k::#ident(#(#idents),*) => #enum_ident::#ident(#(#conversions),*),}
				},
			}
		});

		let cast_variant = variant_idx.iter().copied().filter_map(|u| stripped_variants.get(u)).map(|v| {
			let ident = &v.ident;
			match &v.fields {
				Fields::Named(named) => {
					let idents = named.named.iter().flat_map(|f| {f.ident.as_ref()}).collect::<Vec<_>>();
					let conversions = named.named.iter().map(|f| if find_generic_candidates(&f.ty, |ident| ident == "Self").is_empty() {
						let ident = f.ident.as_ref();
						quote! {#ident}
					} else {
						let ident = f.ident.as_ref();
						quote! {#ident: #ident.cast()?}
					});
					quote! {#enum_ident::#ident{#(#idents),*} => Some(#k::#ident{#(#conversions),*}),}
				},
				Fields::Unit => quote! {#enum_ident::#ident => Some(#k::#ident),},
				Fields::Unnamed(unnamed) => {
					let idents = unnamed.unnamed.iter().enumerate().map(|(idx, f)| {
						Ident::new(&format!("e_{idx}"), f.span())
					}).collect::<Vec<_>>();
					let conversions = unnamed.unnamed.iter().enumerate().map(|(idx, f)| {
						if find_generic_candidates(&f.ty, |ident| ident == "Self").is_empty() {
							let ident = Ident::new(&format!("e_{idx}"), f.span());
							quote! {#ident}
						} else {
							let ident = Ident::new(&format!("e_{idx}"), f.span());
							quote! {#ident.cast()?}
						}
					});
					quote! {#enum_ident::#ident(#(#idents),*) => Some(#k::#ident(#(#conversions),*)),}
				},
			}
		});

		let (_impl_generics, ty_generics, where_clause) = generics.split_for_impl();

		let cross_cast = sub_type_map.iter().filter(|(k2, _v2)| {
			k != *k2
		}).map(|(k2, (variant_idx2, generics2))| {
			let cast_variant = variant_idx.intersection(variant_idx2).copied().filter_map(|u| stripped_variants.get(u)).map(|v| {
				let ident = &v.ident;
				match &v.fields {
					Fields::Named(named) => {
						let idents = named.named.iter().flat_map(|f| {f.ident.as_ref()}).collect::<Vec<_>>();
						let conversions = named.named.iter().map(|f| if find_generic_candidates(&f.ty, |ident| ident == "Self").is_empty() {
							let ident = f.ident.as_ref();
							quote! {#ident}
						} else {
							let ident = f.ident.as_ref();
							quote! {#ident: #ident.cast()?}
						});
						quote! {#k::#ident{#(#idents),*} => Some(#k2::#ident{#(#conversions),*}),}
					},
					Fields::Unit => quote! {#k::#ident => Some(#k2::#ident),},
					Fields::Unnamed(unnamed) => {
						let idents = unnamed.unnamed.iter().enumerate().map(|(idx, f)| {
							Ident::new(&format!("e_{idx}"), f.span())
						}).collect::<Vec<_>>();
						let conversions = unnamed.unnamed.iter().enumerate().map(|(idx, f)| {
							if find_generic_candidates(&f.ty, |ident| ident == "Self").is_empty() {
								let ident = Ident::new(&format!("e_{idx}"), f.span());
								quote! {#ident}
							} else {
								let ident = Ident::new(&format!("e_{idx}"), f.span());
								quote! {#ident.cast()?}
							}
						});
						quote! {#k::#ident(#(#idents),*) => Some(#k2::#ident(#(#conversions),*)),}
					},
				}
			});

			let mut merged_generics = generics.clone();
			let generics_set = generics.params.iter().map(|p| match p {
				GenericParam::Const(c) => &c.ident,
				GenericParam::Lifetime(lt) => &lt.lifetime.ident,
				GenericParam::Type(ty) => &ty.ident,
			}).collect::<HashSet<_>>();
			for g in generics2.params.iter().filter(|p| match p {
				GenericParam::Const(c) => !generics_set.contains(&c.ident),
				GenericParam::Lifetime(lt) => !generics_set.contains(&lt.lifetime.ident),
				GenericParam::Type(ty) => !generics_set.contains(&ty.ident),
			}).cloned() {
				merged_generics.params.push(g);
			}
			merged_generics.where_clause = derive_input.generics.where_clause.clone();
			let (_impl_generics2, ty_generics2, _where_clause2) = generics2.split_for_impl();
			let (merged_impl_generics, _merged_ty_generics, merged_where_clause) = merged_generics.split_for_impl();

			quote! {
				impl #merged_impl_generics PolyEnum<#k2 #ty_generics2> for #k #ty_generics #merged_where_clause {
					fn cast(self) -> Option<#k2 #ty_generics2> {
						match self {
							#(#cast_variant)*
							_ => None,
						}
					}
				}
			}
		});

		let derive_tokens = derive_input.attrs.iter().filter(|attr| {
			if let Some(ident) = attr.meta.path().get_ident() {
				ident == "poly_derive"
			} else {false}
		}).filter_map(|attr| {
			attr.meta.require_list().ok()
		}).flat_map(|attr| attr.tokens.clone().into_iter());

		let (parent_impl_generics, parent_ty_generics, parent_where_clause) = derive_input.generics.split_for_impl();
		let vis = derive_input.vis.clone();

		quote! {
			#[derive(#(#derive_tokens)*)]
			#vis enum #k #ty_generics #where_clause {#(#variants),*}

			impl #parent_impl_generics PolyEnum<#k #ty_generics> for #enum_ident #parent_ty_generics #parent_where_clause {
				fn cast(self) -> Option<#k #ty_generics> {
					match self {
						#(#cast_variant)*
						_ => None,
					}
				}
			}

			impl #parent_impl_generics From<#k #ty_generics> for #enum_ident #parent_ty_generics #parent_where_clause {
				fn from(value: #k #ty_generics) -> #enum_ident #parent_ty_generics {
					match value {
						#(#from_variant)*
					}
				}
			}

			impl #parent_impl_generics PolyEnum<#enum_ident #parent_ty_generics> for #k #ty_generics #parent_where_clause {
				#[inline]
				fn cast(self) -> Option<#enum_ident #parent_ty_generics> {
					Some(self.into())
				}
			}

			#(#cross_cast)*
		}
	});

	quote! {
		#(#sub_types)*
	}.into()
}
