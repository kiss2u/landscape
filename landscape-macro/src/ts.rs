use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, DeriveInput};

pub fn export_ts_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let enum_ident = &input.ident;
    let enum_name = format!("{}TsEnum", enum_ident);
    let enum_ident_ts = syn::Ident::new(&enum_name, enum_ident.span());

    let copied_attrs = input.attrs.iter().filter(|attr| {
        if attr.path().is_ident("ts") {
            return true;
        }

        should_process_attr(&attr)
    });

    let data = match &input.data {
        syn::Data::Enum(e) => e,
        _ => panic!("ExportTsEnum only supports enum"),
    };

    let variants = data.variants.iter().map(|v| {
        let ident = &v.ident;
        quote! {
            #ident
        }
    });

    let expanded = quote! {
        #[derive(::serde::Serialize, ::serde::Deserialize, ::ts_rs::TS)]
        #(#copied_attrs)*
        pub enum #enum_ident_ts {
            #(#variants),*
        }
    };

    expanded.into()
}

fn should_process_attr(attr: &Attribute) -> bool {
    if attr.path().is_ident("serde") {
        let mut has_tag = false;

        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("tag") {
                has_tag = true;
            }
            Ok(())
        });

        return !has_tag;
    }

    true
}
