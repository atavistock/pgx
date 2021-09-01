use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    parse::{Parse, ParseStream},
    DeriveInput, Ident, ItemEnum, ItemStruct,
};

/// A parsed `#[derive(PostgresOrd)]` item.
///
/// It should be used with [`syn::parse::Parse`] functions.
///
/// Using [`quote::ToTokens`] will output the declaration for a `pgx::datum::inventory::InventoryPostgresOrd`.
///
/// On structs:
///
/// ```rust
/// use syn::{Macro, parse::Parse, parse_quote, parse};
/// use quote::{quote, ToTokens};
/// use pgx_utils::inventory::PostgresOrd;
///
/// # fn main() -> eyre::Result<()> {
/// let parsed: PostgresOrd = parse_quote! {
///     #[derive(PostgresOrd)]
///     struct Example<'a> {
///         demo: &'a str,
///     }
/// };
/// let inventory_tokens = parsed.to_token_stream();
/// # Ok(())
/// # }
/// ```
///
/// On enums:
///
/// ```rust
/// use syn::{Macro, parse::Parse, parse_quote, parse};
/// use quote::{quote, ToTokens};
/// use pgx_utils::inventory::PostgresOrd;
///
/// # fn main() -> eyre::Result<()> {
/// let parsed: PostgresOrd = parse_quote! {
///     #[derive(PostgresOrd)]
///     enum Demo {
///         Example,
///     }
/// };
/// let inventory_tokens = parsed.to_token_stream();
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct PostgresOrd {
    pub name: Ident,
}

impl PostgresOrd {
    pub fn new(name: Ident) -> Self {
        Self { name }
    }

    pub fn from_derive_input(derive_input: DeriveInput) -> Result<Self, syn::Error> {
        Ok(Self::new(derive_input.ident))
    }
}

impl Parse for PostgresOrd {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let parsed_enum: Result<ItemEnum, syn::Error> = input.parse();
        let parsed_struct: Result<ItemStruct, syn::Error> = input.parse();
        let ident = parsed_enum
            .map(|x| x.ident)
            .or_else(|_| parsed_struct.map(|x| x.ident))
            .map_err(|_| syn::Error::new(input.span(), "expected enum or struct"))?;
        Ok(Self::new(ident))
    }
}

impl ToTokens for PostgresOrd {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let name = &self.name;
        let inventory_fn_name = syn::Ident::new(
            &format!("__pgx_internals_ord_{}", self.name),
            Span::call_site(),
        );
        let pg_finfo_fn_name = syn::Ident::new(
            &format!("pg_finfo_{}_wrapper", inventory_fn_name),
            Span::call_site(),
        );
        let inv = quote! {
            #[no_mangle]
            pub extern "C" fn  #pg_finfo_fn_name() -> &'static pg_sys::Pg_finfo_record {
                const V1_API: pg_sys::Pg_finfo_record = pg_sys::Pg_finfo_record { api_version: 1 };
                &V1_API
            }

            #[no_mangle]
            #[link(kind = "static")]
            pub extern "C" fn  #inventory_fn_name() -> pgx::datum::inventory::SqlGraphEntity {
                use core::any::TypeId;
                let submission = pgx::datum::inventory::InventoryPostgresOrd {
                    name: stringify!(#name),
                    file: file!(),
                    line: line!(),
                    full_path: core::any::type_name::<#name>(),
                    module_path: module_path!(),
                    id: TypeId::of::<#name>(),
                };
                pgx::datum::inventory::SqlGraphEntity::Ord(submission)
            }
        };
        tokens.append_all(inv);
    }
}
