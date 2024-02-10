use ::quote::{quote, ToTokens};

use syn::parse_macro_input;
use syn::{DataEnum, Field, Fields, FieldsNamed, FieldsUnnamed, Type};
use syn::{DeriveInput, Error};

use proc_macro::TokenStream as V1TokenStream;

use proc_macro2::{Ident, Span, TokenStream};

use proc_macro_crate::{crate_name, FoundCrate};

type Result<T> = std::result::Result<T, Error>;

// inspired from https://github.com/serde-rs/serde/blob/master/serde_derive

#[proc_macro_derive(TableTempl, attributes(len, index, unique))]
pub fn derive_table_type(input: V1TokenStream) -> V1TokenStream {
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

	expand(&input, &name).unwrap_or_else(to_compile_error)
}

fn to_compile_error(error: syn::Error) -> V1TokenStream {
	let compile_error = syn::Error::to_compile_error(&error);
	quote!(#compile_error).into()
}

macro_rules! err {
	($input:ident, $msg:expr) => {
		Error::new_spanned($input.into_token_stream(), $msg)
	};
}

fn expand(
	input: &DeriveInput,
	name: &TokenStream,
) -> Result<proc_macro::TokenStream> {
	Ok(match &input.data {
		syn::Data::Enum(data) => parse_enum(input, data, name)?.into(),
		syn::Data::Struct(data) => match &data.fields {
			Fields::Named(fields) => {
				let ident = &input.ident;

				let (len, info_block, data_block, from_block) =
					parse_named_fields(fields, name)?;

				let table = quote!(#name::table);
				quote!(
					impl #table::TableTemplate for #ident {
						fn table_info() -> #table::Info {
							{ #info_block }
						}
						fn to_data(&self) -> Vec<#table::column::ColumnData<'_>> {
							use #table::column::ColumnType;
							{ #data_block }
						}
						fn from_data(
							data: Vec<#table::column::ColumnData>
						) -> std::result::Result<Self, #table::column::FromDataError> {
							use #table::column::ColumnType;
							if data.len() != #len {
								return Err(#table::column::FromDataError::Custom(
									"TableTemplate from_data: data isn't long enough"
								))
							}
							let mut data = data.into_iter();
							{ #from_block }
						}
					}
				)
				.into()
			}
			Fields::Unnamed(fields) => {
				parse_unnamed_fields(input, fields, name)?.into()
			}
			f => return Err(err!(f, "not supported")),
		},
		_ => return Err(err!(input, "is not supported")),
	})
}

fn parse_named_fields(
	fields: &FieldsNamed,
	name: &TokenStream,
) -> Result<(usize, TokenStream, TokenStream, TokenStream)> {
	let len = fields.named.len();
	let mut info_stream = quote!(
		let mut info = #name::table::Info::with_capacity(#len);
	);
	let mut data_stream = quote!(
		let mut data = Vec::with_capacity(#len);
	);
	let mut from_stream = quote!();

	for field in fields.named.iter() {
		let (col, data, from) = parse_named_field(field, name)?;
		info_stream.extend(quote!(info.push(#col);));
		data_stream.extend(quote!(data.push(#data);));
		from_stream.extend(from);
	}

	info_stream.extend(quote!(info));
	data_stream.extend(quote!(data));

	Ok((
		len,
		info_stream,
		data_stream,
		quote!(Ok(Self {#from_stream})),
	))
}

fn parse_named_field(
	field: &Field,
	#[allow(unused_variables)] crate_name: &TokenStream,
) -> Result<(TokenStream, TokenStream, TokenStream)> {
	let ident = &field.ident;
	let name = field.ident.as_ref().unwrap().to_string(); // TODO

	let mut len = quote!(None);
	let table = quote!(#crate_name::table);
	let index_kind = quote!(#table::column::IndexKind);
	let mut index = quote!(#index_kind::None);

	// this is name: Type
	// should build Attributes
	// println!("parse named ident: {:?} attr: {:?}", name, field.attrs);
	for attr in &field.attrs {
		match &attr.path {
			p if p.is_ident("len") => {
				let res: syn::LitInt = attr.parse_args()?;
				len = quote!(Some(#res));
			}
			p if p.is_ident("index") => {
				let res: syn::Ident = attr.parse_args()?;
				let index_str = res.to_string();
				index = match index_str.as_str() {
					"primary" => quote!(#index_kind::Primary),
					"unique" => quote!(#index_kind::Unique),
					"index" => quote!(#index_kind::Index),
					_ => return Err(err!(res, "not supported index type")),
				};
			}
			p if p.is_ident("unique") => {
				let res: syn::Ident = attr.parse_args()?;
				let index_str = res.to_string();
				index = quote!(#index_kind::NamedUnique(#index_str));
			}
			_ => {}
		}
	}

	let ty = match &field.ty {
		Type::Path(t) => t,
		t => return Err(err!(t, "only type path is supported")),
	};

	let col = quote!(#table::column::Column::new::<#ty>(#name, #len, #index));
	let data = quote!(self.#ident.to_data());
	let from = quote!(#ident: #ty::from_data(data.next().unwrap())?,);

	Ok((col, data, from))
}

fn parse_unnamed_fields(
	input: &DeriveInput,
	fields: &FieldsUnnamed,
	name: &TokenStream,
) -> Result<TokenStream> {
	if fields.unnamed.len() != 1 {
		return Err(err!(fields, "only single unamed fied supported"));
	}
	let field = fields.unnamed.iter().next().unwrap();

	let ident = &input.ident;
	let ty = &field.ty;

	let table = quote!(#name::table);
	Ok(quote!(
		impl #table::column::ColumnType for #ident {
			fn column_kind() -> #table::column::ColumnKind {
				<#ty as #table::column::ColumnType>::column_kind()
			}
			fn to_data(&self) -> #table::column::ColumnData<'_> {
				self.0.to_data()
			}
			fn from_data(
				data: #table::column::ColumnData
			) -> std::result::Result<Self, #table::column::FromDataError> {
				Ok(Self(<#ty as #table::column::ColumnType>::from_data(data)?))
			}
		}
	))
}

fn parse_enum(
	input: &DeriveInput,
	data: &DataEnum,
	name: &TokenStream,
) -> Result<TokenStream> {
	let mut into_stream = quote!();
	let mut from_stream = quote!();

	for variant in data.variants.iter() {
		if variant.fields != Fields::Unit {
			return Err(err!(variant, "only unit are allowed"));
		}
		let ident = &variant.ident;
		let ident_str = ident.to_string();
		into_stream.extend(quote!(Self::#ident => #ident_str,));
		from_stream.extend(quote!(#ident_str => Ok(Self::#ident),));
	}

	let ident = &input.ident;

	let table = quote!(#name::table);
	Ok(quote!(
		impl #ident {
			pub fn as_str(&self) -> &'static str {
				match self {
					#into_stream
				}
			}
			pub fn from_str(
				s: &str
			) -> std::result::Result<Self, #table::column::FromDataError> {
				match s {
					#from_stream
					_ => Err(#table::column::FromDataError::Custom(
						"text doesnt match any enum variant"
					))
				}
			}
		}

		impl #table::column::ColumnType for #ident {
			fn column_kind() -> #table::column::ColumnKind {
				#table::column::ColumnKind::Text
			}
			fn to_data(&self) -> #table::column::ColumnData {
				#table::column::ColumnData::Text(self.as_str().into())
			}
			fn from_data(
				data: #table::column::ColumnData
			) -> std::result::Result<Self, #table::column::FromDataError> {
				match data {
					#table::column::ColumnData::Text(t) => {
						Self::from_str(t.as_str())
					},
					_ => Err(#table::column::FromDataError::ExpectedType(
						"text for enum"
					))
				}
			}
		}
	))
}
