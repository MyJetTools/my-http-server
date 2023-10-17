use proc_macro2::TokenStream;
use quote::quote;
use types_reader::PropertyType;

use crate::input_models::{http_input_props::get_cache_headers, InputField};

pub fn generate_read_body(input_fields: &[InputField]) -> Result<TokenStream, syn::Error> {
    let data_src = quote!(__reader);

    let mut validations = Vec::with_capacity(input_fields.len());

    let mut reading_fields = Vec::with_capacity(input_fields.len());

    for input_field in input_fields {
        reading_fields.push(generate_reading(input_field, &data_src)?);
        if let Some(validator) = input_field.get_validator()? {
            validations.push(validator);
        }
    }

    let init_fields = super::utils::get_fields_to_read(input_fields)?;

    let cache_headers = get_cache_headers(input_fields);

    let result = quote! {
        let #init_fields ={
            let __body = ctx.request.get_body(#cache_headers).await?;
            let __reader = __body.get_body_data_reader()?;
            #(#reading_fields)*
            #init_fields
        };

        #(#validations)*
    };

    Ok(result)
}

fn generate_reading(
    input_field: &InputField,
    data_src: &TokenStream,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let input_field_name = input_field.get_input_field_name()?;
    match &input_field.property.ty {
        PropertyType::OptionOf(sub_type) => {
            super::utils::verify_default_value(input_field, &sub_type)?;

            let sub_type = sub_type.get_token_stream();

            let default_value = input_field.get_default_value_opt_case()?;

            let let_field_name = input_field.get_let_input_param();

            let line = quote::quote! {
                let #let_field_name = if let Some(value) = #data_src.get_optional(#input_field_name){
                    let value: #sub_type = value.try_into()?;
                    Some(value)
                }else{
                    #default_value
                };
            };

            return Ok(line);
        }
        PropertyType::Struct(..) => {
            if let Some(default_value) = input_field.get_default_value()? {
                if default_value.has_value() {
                    let value = default_value.unwrap_value()?;
                    return default_value.throw_error(
                        format!(
                            "Please use default without value '{}'. Struct or Enum should implement create_default and default value is going to be read from there",
                            value.get_any_value_as_str()?
                        ),
                    );
                }

                let default_value = input_field.get_default_value_opt_case()?;

                let let_field_name = input_field.get_let_input_param();

                let result = quote::quote! {
                   let #let_field_name = match #data_src.get_optional(#input_field_name){
                    Some(value) =>{
                        let value = my_http_server::InputParamValue::from(value);
                        value.try_into()?
                    },
                    None => {
                        #default_value
                    }
                   };

                };

                return Ok(result);
            }

            let result = generate_reading_required(&input_field, &data_src)?;
            return Ok(result);
        }
        _ => {
            super::utils::verify_default_value(input_field, &input_field.property.ty)?;

            if input_field.has_default_value() {
                let default_value = input_field.get_default_value_non_opt_case()?;

                let let_field_name = input_field.get_let_input_param();
                let result = quote::quote! {
                   let #let_field_name = match #data_src.get_optional(#input_field_name){
                    Some(value) =>{
                        let value = my_http_server::InputParamValue::from(value);
                        value.try_into()?
                    },
                    None => {
                        #default_value
                    }
                   };

                };
                return Ok(result);
            }

            let result = generate_reading_required(&input_field, &data_src)?;
            return Ok(result);
        }
    }
}

fn generate_reading_required(
    input_field: &InputField,
    data_src: &TokenStream,
) -> Result<TokenStream, syn::Error> {
    let input_field_name = input_field.get_input_field_name()?;
    let let_field_name = input_field.get_let_input_param();
    Ok(quote!(let #let_field_name = #data_src.get_required(#input_field_name)?.try_into()?;))
}
