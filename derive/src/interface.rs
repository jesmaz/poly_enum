

#[proc_macro_attribute]
pub fn interface(_attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let mut fn_def = match parse::<ItemFn>(item) {
		Ok(e) => e,
		Err(err) => {
			return err.into_compile_error().into();
		},
	};

	let overloads = fn_def.block.stmts.extract_if(0.., |stmt| {
		matches!(stmt, Stmt::Item(Item::Fn(f)) if f.sig.ident == fn_def.sig.ident)
	}).filter_map(|stmt| match stmt {
		Stmt::Item(Item::Fn(f)) => Some(f),
		_ => unreachable!()
	}).collect::<Vec<_>>();

	// fn_def.block.stmts.push(Stmt::Expr(fn_def.span(), Expr::Match(ExprMatch {
	// 	attrs: Default::default(),
	// 	match_token: Default::default(),
	// 	expr: Expr::Tuple(ExprTuple {
	// 		attrs: Default::default(),
	// 		paren_token: Default::default(),

	// 	})
	// })));

	let params = fn_def.sig.inputs.iter().map(|input| match input {
		FnArg::Receiver(recv) => Ident::new("self", recv.span()),
		FnArg::Typed(typed) => match &*typed.pat {
			Pat::Ident(ident) => ident.ident.clone(),
			_ => todo!()
		}
	});
	let cases = overloads.into_iter().map(|mut fn_def| {
		//fn_def.sig.inputs
	});
	let match_expr = quote_spanned! {fn_def.span()=>
		match (#(#params,)*) {}
	};

	let s = quote! {#fn_def}.to_string();
	//Error::new(fn_def.span(), s).into_compile_error().into()
	quote! {#fn_def}.into()
}
