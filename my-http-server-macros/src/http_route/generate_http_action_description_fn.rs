use std::str::FromStr;

use proc_macro2::TokenStream;

use super::action_builder::ActionParameters;

pub fn generate_http_action_description_fn(
    action_parameters: &ActionParameters,
) -> Result<TokenStream, syn::Error> {
    let api_data = action_parameters.get_api_data();
    if api_data.is_none() {
        return Ok(quote::quote!(None));
    }

    let api_data = api_data.unwrap();

    let should_be_authorized = action_parameters.get_should_be_authorized()?;

    let use_documentation = crate::consts::get_use_documentation();

    let http_action_description = crate::consts::get_http_action_description();

    let controller_name = api_data.controller;
    let summary = api_data.summary;
    let description = api_data.description;
    let deprecated = api_data.deprecated;

    let input_params = generate_get_input_params(action_parameters.input_data);

    let results = super::result_model_generator::generate(&api_data.results);

    Ok(quote::quote! {
        #use_documentation;

        #http_action_description{
            controller_name: #controller_name,
            summary: #summary,
            description: #description,
            should_be_authorized: #should_be_authorized,
            input_params: #input_params,
            results: #results,
            deprecated: #deprecated
        }.into()

    })
}

fn generate_get_input_params(input_data: Option<&str>) -> TokenStream {
    if let Some(input_data) = input_data {
        let input_data = TokenStream::from_str(input_data).unwrap();
        quote::quote!(#input_data::get_input_params().into())
    } else {
        quote::quote!(in_parameters::HttpParameters::new(None))
    }
}
