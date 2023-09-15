use proc_macro2::TokenStream;

use super::enum_json::EnumJson;

pub fn generate(enum_cases: &[EnumJson]) -> Result<Vec<TokenStream>, syn::Error> {
    let mut has_default_value = false;

    let mut result = Vec::new();

    for enum_case in enum_cases {
        if enum_case.is_default_value {
            has_default_value = true;
            continue;
        }

        let id = proc_macro2::Literal::isize_unsuffixed(enum_case.get_id()?);

        let enum_case = enum_case.src.get_name_ident();

        let line = quote::quote! {
            #id => Self::#enum_case,
        };
        result.push(line);
    }

    if has_default_value {
        let line = quote::quote!(_ => Self::default());
        result.push(line);
    } else {
        let line = quote::quote!(_ => panic!("Can not parse enum with value {}", src));
        result.push(line);
    }

    Ok(result)
}
