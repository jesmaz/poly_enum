use std::collections::{HashMap, HashSet};

use quote::quote;
use syn::{parse, spanned::Spanned, Expr, ExprLit, Fields, GenericParam, Generics, Ident, ItemEnum, Lit, LitInt};

use crate::util::{find_generic_candidates, parse_attr_variants};

pub fn poly_enum(_attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let mut enum_def = match parse::<ItemEnum>(item) {
		Ok(enum_def) => enum_def,
		Err(err) => return err.into_compile_error().into(),
	};

	let Some(repr_attr) = enum_def.attrs.iter().find(|a| {
		a.meta.path().get_ident().map(|ident| ident == "repr").unwrap_or(false)
	}).cloned() else {
		return quote! {compile_error!("A repr attribute is required, eg: #[repr(C)]")}.into()
	};

	let repr_ty = match repr_attr.parse_args::<Ident>() {
		Ok(repr_ty) if repr_ty == "C" => Ident::new("i32", repr_ty.span()),
		Ok(repr_ty) => repr_ty,
		Err(err) => return err.into_compile_error().into(),
	};

	enum_def.variants.iter_mut().enumerate().for_each(|(idx, v)| v.discriminant = Some((Default::default(), Expr::Lit(ExprLit {
		attrs: Default::default(),
		lit: Lit::Int(LitInt::new(&format!("{idx}"), v.span()))
	}))));

	let mut sub_type_map: HashMap<_, HashSet<_>> = HashMap::new();
	let mut stripped_variants = Vec::with_capacity(enum_def.variants.len());
	for variant in &mut enum_def.variants {
		match variant.attrs.extract_if(0.., |attr| if let Some(ident) = attr.meta.path().get_ident() {
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
		let generics_set = enum_def.generics.params.iter().map(|p| match p {
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
			generics.where_clause = enum_def.generics.where_clause.clone();
		}
		(k, (variant_idx, generics))
	}).collect::<HashMap<_, _>>();

	let (parent_impl_generics, parent_ty_generics, parent_where_clause) = enum_def.generics.split_for_impl();

	let sub_types = sub_type_map.iter().map(|(k, (variant_idx, generics))| {
		let variants = variant_idx.iter().copied().filter_map(|u| stripped_variants.get(u));
		let enum_ident = &enum_def.ident;
		let (_impl_generics, ty_generics, where_clause) = generics.split_for_impl();

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

		let cast_variant_mut = variant_idx.iter().copied().filter_map(|u| stripped_variants.get(u)).map(|v| {
			let transmute = quote! {
				unsafe {::std::mem::transmute::<&mut #enum_ident #parent_ty_generics, &mut #k #ty_generics>(self)}
			};
			let ident = &v.ident;
			match &v.fields {
				Fields::Named(named) => {
					if named.named.iter().all(|f| find_generic_candidates(&f.ty, |ident| ident == "Self").is_empty()) {
						quote! {#enum_ident::#ident{..} => Some(#transmute),}
					} else {
						Default::default()
					}
				},
				Fields::Unit => quote! {#enum_ident::#ident => Some(#transmute),},
				Fields::Unnamed(unnamed) => {
					if unnamed.unnamed.iter().all(|f| find_generic_candidates(&f.ty, |ident| ident == "Self").is_empty()) {
						quote! {#enum_ident::#ident(..) => Some(#transmute),}
					} else {
						Default::default()
					}
				},
			}
		});

		let cast_variant_ref = variant_idx.iter().copied().filter_map(|u| stripped_variants.get(u)).map(|v| {
			let transmute = quote! {
				unsafe {::std::mem::transmute::<&#enum_ident #parent_ty_generics, &#k #ty_generics>(self)}
			};
			let ident = &v.ident;
			match &v.fields {
				Fields::Named(named) => {
					if named.named.iter().all(|f| find_generic_candidates(&f.ty, |ident| ident == "Self").is_empty()) {
						quote! {#enum_ident::#ident{..} => Some(#transmute),}
					} else {
						Default::default()
					}
				},
				Fields::Unit => quote! {#enum_ident::#ident => Some(#transmute),},
				Fields::Unnamed(unnamed) => {
					if unnamed.unnamed.iter().all(|f| find_generic_candidates(&f.ty, |ident| ident == "Self").is_empty()) {
						quote! {#enum_ident::#ident(..) => Some(#transmute),}
					} else {
						Default::default()
					}
				},
			}
		});

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
			merged_generics.where_clause = enum_def.generics.where_clause.clone();
			let (_impl_generics2, ty_generics2, _where_clause2) = generics2.split_for_impl();
			let (merged_impl_generics, _merged_ty_generics, merged_where_clause) = merged_generics.split_for_impl();

			quote! {
				impl #merged_impl_generics ::poly_enum::Cast<#k2 #ty_generics2> for #k #ty_generics #merged_where_clause {
					fn cast(self) -> Option<#k2 #ty_generics2> {
						use ::poly_enum::Cast;
						match self {
							#(#cast_variant)*
							_ => None,
						}
					}
				}
			}
		});

		let derive_tokens = enum_def.attrs.iter().filter(|attr| {
			if let Some(ident) = attr.meta.path().get_ident() {
				ident == "derive"
			} else {false}
		}).filter_map(|attr| {
			attr.meta.require_list().ok()
		}).flat_map(|attr| attr.tokens.clone().into_iter());
		let vis = enum_def.vis.clone();

		let field_assertions = variant_idx.iter().copied().filter_map(|u| stripped_variants.get(u)).map(|v| {
			let ident = &v.ident;
			match &v.fields {
				Fields::Named(named) => {
					let idents = named.named.iter().flat_map(|f| {f.ident.as_ref()}).collect::<Vec<_>>();
					let idents_a = named.named.iter().flat_map(|f| {f.ident.as_ref()}).map(|ident| {
						Ident::new(&format!("{ident}_a"), ident.span())
					}).collect::<Vec<_>>();
					let idents_b = named.named.iter().flat_map(|f| {f.ident.as_ref()}).map(|ident| {
						Ident::new(&format!("{ident}_b"), ident.span())
					}).collect::<Vec<_>>();

					let discriminant = v.discriminant.as_ref().map(|(_, expr)| quote! {#expr}).unwrap_or_else(|| {
						quote! {compile_error!("")}
					});
					quote! {
						const {
							let mut uninit_main_enum = ::std::mem::MaybeUninit::<#enum_ident #parent_ty_generics>::uninit();
							let mut uninit_sub_class_enum = ::std::mem::MaybeUninit::<#k #ty_generics>::uninit();
							unsafe {(uninit_main_enum.as_mut_ptr() as *mut #repr_ty).write(#discriminant)};
							unsafe {(uninit_sub_class_enum.as_mut_ptr() as *mut #repr_ty).write(#discriminant)};
							match unsafe {(uninit_main_enum.assume_init_ref(), uninit_sub_class_enum.assume_init_ref())} {
								(#enum_ident::#ident{#(#idents: #idents_a),*}, #k::#ident{#(#idents: #idents_b),*}) => {
									#(
										let a_offset = unsafe {uninit_main_enum.as_ptr().byte_offset_from(#idents_a)};
										let b_offset = unsafe {uninit_sub_class_enum.as_ptr().byte_offset_from(#idents_b)};
										assert!(a_offset == b_offset);
									)*
								},
								_ => unreachable!(),
							};
						};
					}
				},
				Fields::Unit => Default::default(),
				Fields::Unnamed(unnamed) => {
					let idents_a = unnamed.unnamed.iter().enumerate().map(|(idx, f)| {
						Ident::new(&format!("e_{idx}_a"), f.span())
					}).collect::<Vec<_>>();
					let idents_b = unnamed.unnamed.iter().enumerate().map(|(idx, f)| {
						Ident::new(&format!("e_{idx}_b"), f.span())
					}).collect::<Vec<_>>();

					let discriminant = v.discriminant.as_ref().map(|(_, expr)| quote! {#expr}).unwrap_or_else(|| {
						quote! {compile_error!("")}
					});
					quote! {
						const {
							let mut uninit_main_enum = ::std::mem::MaybeUninit::<#enum_ident #parent_ty_generics>::uninit();
							let mut uninit_sub_class_enum = ::std::mem::MaybeUninit::<#k #ty_generics>::uninit();
							unsafe {(uninit_main_enum.as_mut_ptr() as *mut #repr_ty).write(#discriminant)};
							unsafe {(uninit_sub_class_enum.as_mut_ptr() as *mut #repr_ty).write(#discriminant)};
							match unsafe {(uninit_main_enum.assume_init_ref(), uninit_sub_class_enum.assume_init_ref())} {
								(#enum_ident::#ident(#(#idents_a),*), #k::#ident(#(#idents_b),*)) => {
									#(
										let a_offset = unsafe {uninit_main_enum.as_ptr().byte_offset_from(#idents_a)};
										let b_offset = unsafe {uninit_sub_class_enum.as_ptr().byte_offset_from(#idents_b)};
										assert!(a_offset == b_offset);
									)*
								},
								_ => unreachable!(),
							};
						};
					}
				},
			}
		});

		quote! {
			#[derive(#(#derive_tokens)*)]
			#repr_attr
			#vis enum #k #ty_generics #where_clause {#(#variants),*}

			impl #parent_impl_generics ::poly_enum::Cast<#k #ty_generics> for #enum_ident #parent_ty_generics #parent_where_clause {
				fn cast(self) -> Option<#k #ty_generics> {
					use ::poly_enum::Cast;
					#(#field_assertions)*
					match self {
						#(#cast_variant)*
						_ => None,
					}
				}
			}

			impl #parent_impl_generics ::poly_enum::CastRef<#k #ty_generics> for #enum_ident #parent_ty_generics #parent_where_clause {
				fn cast_mut(&mut self) -> Option<&mut #k #ty_generics> {
					use ::poly_enum::CastRef;
					match self {
						#(#cast_variant_mut)*
						_ => None,
					}
				}
				fn cast_ref(&self) -> Option<&#k #ty_generics> {
					use ::poly_enum::CastRef;
					match self {
						#(#cast_variant_ref)*
						_ => None,
					}
				}
			}

			impl #parent_impl_generics ::std::borrow::Borrow<#enum_ident #parent_ty_generics> for #k #ty_generics #parent_where_clause {
				fn borrow(&self) -> &#enum_ident #parent_ty_generics {
					unsafe {::std::mem::transmute::<&#k #ty_generics, &#enum_ident #parent_ty_generics>(self)}
				}
			}

			impl #parent_impl_generics From<#k #ty_generics> for #enum_ident #parent_ty_generics #parent_where_clause {
				fn from(value: #k #ty_generics) -> #enum_ident #parent_ty_generics {
					use ::poly_enum::Cast;
					match value {
						#(#from_variant)*
					}
				}
			}

			impl #parent_impl_generics ::poly_enum::Cast<#enum_ident #parent_ty_generics> for #k #ty_generics #parent_where_clause {
				#[inline]
				fn cast(self) -> Option<#enum_ident #parent_ty_generics> {
					Some(self.into())
				}
			}

			#(#cross_cast)*
		}
	});

	quote! {
		#enum_def

		#(#sub_types)*
	}.into()
}
