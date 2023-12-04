use proc_macro2::TokenStream;
use types_reader::PropertyType;

use crate::input_models::InputField;

pub fn get_fields_to_read(fields: &[InputField]) -> Result<TokenStream, syn::Error> {
    if fields.len() == 1 {
        let field = fields.get(0).unwrap();
        let name = field.property.get_field_name_ident();

        return Ok(quote::quote!(#name));
    }

    let mut no = 0;

    let mut result = Vec::with_capacity(fields.len());

    for input_field in fields {
        if no > 0 {
            result.push(quote::quote!(,));
        }

        let ident = input_field.property.get_field_name_ident();
        result.push(quote::quote!(#ident));

        no += 1;
    }

    Ok(quote::quote! {(#(#result)*)})
}

pub fn verify_default_value(input_field: &InputField, ty: &PropertyType) -> Result<(), syn::Error> {
    let empty_only = match ty {
        PropertyType::U8 => false,
        PropertyType::I8 => false,
        PropertyType::U16 => false,
        PropertyType::I16 => false,
        PropertyType::U32 => false,
        PropertyType::I32 => false,
        PropertyType::U64 => false,
        PropertyType::I64 => false,
        PropertyType::F32 => false,
        PropertyType::F64 => false,
        PropertyType::USize => false,
        PropertyType::ISize => false,
        PropertyType::String => false,
        PropertyType::Bool => false,
        PropertyType::DateTime => false,
        PropertyType::OptionOf(_) => false,
        PropertyType::VecOf(_) => false,
        PropertyType::Struct(_, _) => true,
        PropertyType::HashMap(_, _) => false,
        PropertyType::RefTo { ty: _, lifetime: _ } => false,
    };

    if empty_only {
        let default_value = input_field.get_default_value()?;
        match default_value {
            Some(default_value) => {
                if !default_value.is_empty() {
                    return default_value.throw_error("Please use default parameter with NO value");
                }

                return Ok(());
            }
            None => return Ok(()),
        }
    } else {
        let default_value = input_field.get_default_value()?;
        match default_value {
            Some(default_value) => {
                if default_value.is_empty() {
                    return default_value.throw_error("Please use default parameter with value");
                }

                return Ok(());
            }
            None => return Ok(()),
        }
    }
}

pub enum ReadParamSrc {
    QueryString,
}

impl ReadParamSrc {
    pub fn to_token_stream(&self) -> TokenStream {
        match self {
            Self::QueryString => quote::quote!(my_http_server::data_src::SRC_QUERY_STRING),
        }
    }
}

pub fn read_param_as_array(
    data_src: &TokenStream,
    input_field_name: &str,
    src: ReadParamSrc,
) -> TokenStream {
    let src_as_token_stream = src.to_token_stream();
    quote::quote! {
        {
            let items = #data_src.get_vec(#input_field_name)?;
            let mut result = Vec::with_capacity(items.len());

            for value in items {
                let encoded = my_http_server::EncodedParamValue::from_url_encoded_data(
                    value,
                    #src_as_token_stream,
                );
                result.push(encoded.try_into()?);
            }

            result
        };
    }
}
