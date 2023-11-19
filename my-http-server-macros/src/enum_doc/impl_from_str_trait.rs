use proc_macro2::TokenStream;

use super::enum_json::EnumJson;

pub fn generate(name: &str, enum_cases: &[EnumJson]) -> Result<Vec<TokenStream>, syn::Error> {
    let mut result = Vec::new();

    let mut default_value = false;
    for enum_case in enum_cases {
        if enum_case.is_default_value {
            default_value = true;
            continue;
        }

        let value = enum_case.get_enum_case_str_value()?;
        let value = value.as_str();

        let enum_id = enum_case.get_enum_case_value();

        let id = enum_case.get_id()?.to_string();
        let enum_value = &enum_case.src.get_name_ident();

        result.push(quote::quote! {
            if src == #value || src == #id || src == #enum_id {
                return Ok(Self::#enum_value)
            }
        });
    }

    if default_value {
        result.push(quote::quote!(Ok(Self::default())));
    } else {
        let http_fail_result = crate::consts::get_http_fail_result();
        let err = format!("Can not parse {} enum", name);
        result.push(quote::quote! {
            Err(#http_fail_result::as_forbidden(Some(#err.to_string())))
        });
    }

    Ok(result)
}
