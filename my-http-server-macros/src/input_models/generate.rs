use proc_macro::TokenStream;

use quote::quote;

use super::http_input_props::HttpInputProperties;

pub fn generate(ast: &syn::DeriveInput, debug: &mut bool) -> Result<TokenStream, syn::Error> {
    let struct_name = &ast.ident;

    let fields = types_reader::StructProperty::read(ast)?;

    for prop in &fields {
        if prop.attrs.has_attr("debug") {
            *debug = true;
        }
    }

    let input_fields = HttpInputProperties::new(&fields)?;

    let print_request_to_console = if input_fields.print_request_to_console {
        quote::quote! {
            {
                let query = ctx.request.get_query_string()?;
                println!("QueryString: '{}'", query.get_raw());
                for (name, value) in ctx.request.get_headers().to_hash_map() {
                    println!("Header: ['{}']: '{}'", name, value);
                }

                let body = ctx.request.get_body().await?;
                println!("===Body Start===");
                print!("{}", body.as_str()?);
                println!("===Body End===");
            }
        }
    } else {
        quote::quote!()
    };

    let http_input_param = crate::consts::get_http_input_parameter_with_ns();

    let http_ctx = crate::consts::get_http_context();

    let http_fail_result = crate::consts::get_http_fail_result();

    let http_input = match super::docs::generate_http_input(&input_fields) {
        Ok(result) => result,
        Err(err) => err.to_compile_error(),
    };

    let parse_http_input = match super::model_reader::generate(&struct_name, &input_fields) {
        Ok(result) => result,
        Err(err) => err.to_compile_error(),
    };

    let http_routes = match http_routes(&input_fields) {
        Ok(result) => {
            if result.is_empty() {
                quote! {None}
            } else {
                quote!(Some(vec![#(#result),*]))
            }
        }
        Err(err) => err.to_compile_error(),
    };

    let result = quote! {
        impl #struct_name{
            pub fn get_input_params()->Vec<#http_input_param>{
                #http_input
            }

            pub async fn parse_http_input(http_route: &my_http_server::controllers::HttpRoute, ctx: &mut #http_ctx)->Result<Self,#http_fail_result>{
                use my_http_server::*;
                #print_request_to_console
                #parse_http_input
            }

            pub fn get_model_routes()->Option<Vec<&'static str>>{
                #http_routes
            }
        }
    };
    Ok(result.into())
}

fn http_routes(props: &HttpInputProperties) -> Result<Vec<proc_macro2::TokenStream>, syn::Error> {
    let mut result = Vec::new();

    if let Some(path_fields) = &props.path_fields {
        for input_field in path_fields {
            let name = input_field.get_input_field_name()?;
            result.push(quote! {
                #name
            });
        }
    }

    Ok(result)
}
