use std::str::FromStr;

use proc_macro::TokenStream;
use types_reader::TokensObject;

use super::attributes::HttpRouteModel;


pub fn build_action(attr: TokenStream, input: TokenStream) -> Result<TokenStream, syn::Error> {

    let ast: syn::DeriveInput = syn::parse(input).unwrap();


    let attr:proc_macro2::TokenStream = attr.into();
    let params_list = TokensObject::new(attr.into(), &||None)?;

    let action_model = HttpRouteModel::parse(&params_list)?;


    let struct_name = &ast.ident;

    let trait_name = action_model.method.get_trait_name();

    let route = action_model.route;

    let http_action_description = crate::consts::get_http_action_description_with_ns();

    let description = super::generate_http_action_description_fn(&action_model)?;

    let http_route = crate::consts::get_http_route();

    let http_context = crate::consts::get_http_context();

    let http_ok_result = crate::consts::get_http_ok_result();

    let http_fail_result = crate::consts::get_http_fail_result();

    let handle_request = super::generate_handle_request_fn(action_model.input_data);

    let model_routes: proc_macro2::TokenStream = if let Some(input_data) = &action_model.input_data{
        let input_data = proc_macro2::TokenStream::from_str(input_data).unwrap();
        quote::quote!(#input_data::get_model_routes())    
    }else{
        quote::quote!(None)
    };

    let result = quote::quote! {
        #ast

        impl #trait_name for #struct_name{
            fn get_route(&self) -> &str {
                #route
            }

            fn get_model_routes(&self) -> Option<Vec<&'static str>>{
                #model_routes
            }
        }

        impl my_http_server::controllers::actions::GetDescription for #struct_name{
            fn get_description(&self) -> Option<#http_action_description>{
                #description
            }
        }

        #[async_trait::async_trait]
        impl my_http_server::controllers::actions::HandleHttpRequest for #struct_name{
            async fn handle_request(&self, http_route: &#http_route, ctx: &mut #http_context) -> Result<#http_ok_result, #http_fail_result> {
                #handle_request
            }
        }
  
    }
    .into();

   Ok(result)
}
