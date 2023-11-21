use quote::quote;
use types_reader::StructProperty;

use crate::generic_utils::GenericData;

pub fn generate(ast: &syn::DeriveInput) -> (proc_macro::TokenStream, bool) {
    let struct_name = &ast.ident;

    let mut debug = false;

    let fields = match StructProperty::read(ast) {
        Ok(result) => result,
        Err(err) => return (err.into_compile_error().into(), debug),
    };

    for prop in &fields {
        if prop.attrs.has_attr("debug") {
            debug = true;
        }
    }

    let generic_data = GenericData::new(ast);

    let (generic, generic_ident) = if let Some(generic) = generic_data.as_ref() {
        let generic_token_stream = generic.generic.clone();
        let generic_ident = generic.generic_ident.clone();

        (generic_token_stream, generic_ident)
    } else {
        (quote::quote! {}, quote::quote! {})
    };

    let get_http_data_structure = match super::generate_get_http_data_structure(
        struct_name,
        generic_data.as_ref(),
        &fields,
    ) {
        Ok(result) => result,
        Err(err) => return (err.into_compile_error().into(), debug),
    };

    let data_structure_provider = match crate::http_object_structure::generate_data_provider(
        struct_name,
        generic_data.as_ref(),
    ) {
        Ok(result) => result,
        Err(err) => return (err.into_compile_error().into(), debug),
    };

    let result = quote! {

        #data_structure_provider

        impl #generic #struct_name #generic_ident {
            #get_http_data_structure
        }

    }
    .into();

    (result, debug)
}
