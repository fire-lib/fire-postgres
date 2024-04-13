mod from_row;
mod table_templ;

use ::quote::quote;
use from_row::expand_from_row;

use syn::parse_macro_input;
use syn::DeriveInput;

use proc_macro::TokenStream as V1TokenStream;

use proc_macro2::{Ident, Span};

use proc_macro_crate::{crate_name, FoundCrate};
use table_templ::expand_table_templ;

// inspired from https://github.com/serde-rs/serde/blob/master/serde_derive

#[proc_macro_derive(TableTempl, attributes(len, index, unique))]
pub fn derive_table_templ(input: V1TokenStream) -> V1TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	// crate name
	let name =
		crate_name("fire-postgres").expect("fire-postgres not in dependencies");
	let name = match name {
		FoundCrate::Itself => quote!(crate),
		FoundCrate::Name(n) => {
			let ident = Ident::new(&n, Span::call_site());
			quote!(#ident)
		}
	};

	expand_table_templ(&input, &name).unwrap_or_else(to_compile_error)
}

// attributes(len, index, unique)
#[proc_macro_derive(FromRow)]
pub fn derive_from_row(input: V1TokenStream) -> V1TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	// crate name
	let name =
		crate_name("fire-postgres").expect("fire-postgres not in dependencies");
	let name = match name {
		FoundCrate::Itself => quote!(crate),
		FoundCrate::Name(n) => {
			let ident = Ident::new(&n, Span::call_site());
			quote!(#ident)
		}
	};

	expand_from_row(&input, &name).unwrap_or_else(to_compile_error)
}

fn to_compile_error(error: syn::Error) -> V1TokenStream {
	let compile_error = syn::Error::to_compile_error(&error);
	quote!(#compile_error).into()
}
