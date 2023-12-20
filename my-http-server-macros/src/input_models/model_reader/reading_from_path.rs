use crate::input_models::InputField;

pub fn reading_from_path(
    input_fields: &[InputField],
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let mut reading_fields = Vec::with_capacity(input_fields.len());
    let mut validations = Vec::with_capacity(input_fields.len());
    for input_field in input_fields {
        if let Some(validation) = input_field.get_validator_as_token_stream() {
            validations.push(validation);
        }

        let let_input_param = input_field.get_let_input_param();

        let input_field_name = input_field.get_input_field_name()?;

        reading_fields.push(quote::quote!(let #let_input_param =  http_route.get_value(&ctx.request.http_path, #input_field_name)?.try_into()?;))
    }

    let result = quote::quote! {
        #(#reading_fields)*
        #(#validations)*
    };

    Ok(result)
}
