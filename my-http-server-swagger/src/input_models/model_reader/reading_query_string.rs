use std::str::FromStr;

use proc_macro2::TokenStream;
use types_reader::PropertyType;

use crate::input_models::InputField;

pub fn generate_reading_query_fields(
    input_fields: &[InputField],
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let mut reading_fields = Vec::with_capacity(input_fields.len());

    let data_src = TokenStream::from_str("___data_src").unwrap();

    let init_fields = super::utils::get_fields_to_read(input_fields)?;

    let mut validations = Vec::with_capacity(input_fields.len());

    for input_field in input_fields {
        let token_stream = reading_query_string(input_field, &data_src)?;

        if let Some(validation) = input_field.get_validator()? {
            validations.push(validation);
        }

        reading_fields.push(token_stream)
    }

    let result = quote::quote! {
        let #init_fields = {
            let #data_src = ctx.request.get_query_string()?;
            #(#reading_fields)*
            #init_fields
        };
        #(#validations)*
    };

    Ok(result)
}

fn reading_query_string(
    input_field: &InputField,
    data_src: &TokenStream,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let input_field_name = input_field.get_input_field_name()?;

    match &input_field.property.ty {
        PropertyType::OptionOf(sub_ty) => {
            super::utils::verify_default_value(input_field, &sub_ty)?;

            let default_value = input_field.get_default_value_opt_case()?;

            let let_input_param = input_field.get_let_input_param();

            let result = quote::quote! {
                let #let_input_param = if let Some(value) = #data_src.get_optional(#input_field_name) {
                    let value = my_http_server::InputParamValue::from(value);
                    Some(value.try_into()?)
                } else {
                    #default_value
                };
            };

            return Ok(result);
        }
        PropertyType::VecOf(sub_type) => {
            let struct_field_name = input_field.property.get_field_name_ident();

            if sub_type.is_string() {
                let input_field_name = input_field.get_input_field_name()?;

                let item = quote::quote! {
                  let #struct_field_name = #data_src.get_vec_of_string(#input_field_name)?;
                }
                .into();

                return Ok(item);
            } else {
                let input_field_name = input_field.get_input_field_name()?;

                let item = quote::quote! {
                   let #struct_field_name = #data_src.get_vec(#input_field_name)?;
                }
                .into();

                return Ok(item);
            }
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

                let let_input_param = input_field.get_let_input_param();

                let result = quote::quote! {
                   let #let_input_param = match #data_src.get_optional(#input_field_name){
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

            return Ok(generate_reading_required(input_field, &data_src)?);
        }
        _ => {
            super::utils::verify_default_value(input_field, &input_field.property.ty)?;

            let let_input_param = input_field.get_let_input_param();

            if input_field.has_default_value() {
                let default_value = input_field.get_default_value_non_opt_case()?;

                let result = quote::quote! {
                   let #let_input_param = match #data_src.get_optional(#input_field_name){
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
            return Ok(generate_reading_required(input_field, &data_src)?);
        }
    }
}

fn generate_reading_required(
    input_field: &InputField,
    data_src: &TokenStream,
) -> Result<TokenStream, syn::Error> {
    let struct_field_name = input_field.property.get_field_name_ident();
    let input_field_name = input_field.get_input_field_name()?;
    if let Some(default_value) = input_field.get_default_value()? {
        match default_value {
            crate::input_models::DefaultValue::Empty(_) => {
                let prop_type = input_field.property.get_syn_type();
                let result = quote::quote!(#prop_type::create_default()?);

                return Ok(result);
            }
            crate::input_models::DefaultValue::Value(default) => {
                let default = default.get_any_value_as_str()?;
                let else_data = proc_macro2::TokenStream::from_str(default);

                if let Err(err) = else_data {
                    return Err(syn::Error::new_spanned(
                        input_field.property.field,
                        format!("Invalid default value: {}", err),
                    ));
                }

                let else_data = else_data.unwrap();

                let let_input_param = input_field.get_let_input_param();

                let result = quote::quote! {
                    let #let_input_param = if let Some(value) = #data_src.get_optional(#input_field_name){
                        my_http_server::InputParamValue::from(value).try_into()?
                    }else{
                        #else_data
                    };
                };

                return Ok(result);
            }
        }
    } else {
        let ty = input_field.property.ty.get_token_stream();
        return Ok(
            quote::quote!(let #struct_field_name: #ty = my_http_server::InputParamValue::from(#data_src.get_required(#input_field_name)?).try_into()?;),
        );
    }
}
