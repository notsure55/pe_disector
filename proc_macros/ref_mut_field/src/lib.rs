use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::*;

#[derive(PartialEq, Eq)]
enum Ref {
    None,
    Ref,
    Mut,
}

#[proc_macro_derive(RefMutFields, attributes(ref_mut))]
pub fn derive(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as DeriveInput);

    let name = parsed.ident.clone();

    let fields: Vec<_> = {
        if let Data::Struct(data) = parsed.data {
            data.fields
                .into_iter()
                .map(|field| {
                    // should only be one attribute
                    if let Some(attr) = field.attrs.first() {
                        if let Some(ident) = attr.path().get_ident() {
                            match ident.to_string().as_ref() {
                                "ref" => (field, Ref::Ref),
                                "ref_mut" => (field, Ref::Mut),
                                _ => (field, Ref::None),
                            }
                        } else {
                            (field, Ref::None)
                        }
                    } else {
                        (field, Ref::None)
                    }
                })
                .collect()
        } else {
            unreachable!("Used a non struct type for proc macro refmutfields")
        }
    };

    let fields_mut: Vec<_> = fields
        .iter()
        .filter_map(|(field, r)| match r {
            Ref::Mut => Some(field),
            _ => None,
        })
        .collect();

    let methods: Vec<_> = fields_mut
        .iter()
        .map(|field| {
            if let Some(ref name) = field.ident {
                let get_name_mut = format_ident!("get_{}_mut", name);
                let get_name = format_ident!("get_{}", name);
                let typ = &field.ty;
                quote! {
                    pub fn #get_name(&self) -> &#typ {
                        &self.#name
                    }
                    pub fn #get_name_mut(&mut self) -> &mut #typ {
                        &mut self.#name
                    }
                }
            } else {
                unreachable!("doesnt handle non named fields")
            }
        })
        .collect();

    let tokens = quote! {
        impl #name {
            #(#methods)*
        }
    };

    tokens.into()
}
