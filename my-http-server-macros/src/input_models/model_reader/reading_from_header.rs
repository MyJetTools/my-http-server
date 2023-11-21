use crate::input_models::InputField;

pub fn reading_from_header(
    input_fields: &[InputField],
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let mut reading_fields = Vec::with_capacity(input_fields.len());
    let mut validations = Vec::with_capacity(input_fields.len());
    for input_field in input_fields {
        if let Some(validation) = input_field.get_validator()? {
            validations.push(validation);
        }

        let let_input_param = input_field.get_let_input_param();

        let header_value = read_header_field(input_field)?;

        reading_fields.push(quote::quote!(let #let_input_param = #header_value ))
    }

    let result = quote::quote! {
        #(#reading_fields)*
        #(#validations)*
    };

    Ok(result)
}

fn read_header_field(input_field: &InputField) -> Result<proc_macro2::TokenStream, syn::Error> {
    let input_field_name = input_field.get_input_field_name()?;

    if input_field.property.ty.is_option() {
        let default_value = input_field.get_default_value_opt_case()?;

        let result = quote::quote! {
            if let Some(value) = ctx.request.get_headers().try_get_case_insensitive(#input_field_name) {
                Some(value.try_into()?)
            } else {
                #default_value
            };
        };

        return Ok(result);
    }

    if !input_field.has_default_value() {
        let result = quote::quote!(ctx.request.get_headers().get_required_case_insensitive(#input_field_name)?.try_into()?;);
        return Ok(result);
    }

    let default_value = input_field.get_default_value_non_opt_case()?;

    let result = quote::quote! {
        if let Some(value) = ctx.request.get_headers().try_get_case_insensitive(#input_field_name) {
            value.try_into()?
        } else {
            #default_value
        };
    };

    return Ok(result);
}
