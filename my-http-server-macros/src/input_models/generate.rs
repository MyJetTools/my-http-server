use proc_macro::TokenStream;

use super::http_input_props::HttpInputProperties;

/// Server-only half of the model derive: generates just `parse_http_input`, which reads the
/// model out of an incoming request. The schema (`get_input_params`/`get_model_routes`) and the
/// client request builder come from `url_utils::macros::MyHttpInput`; this one is added on the
/// server via `MyHttpInputServer` so the same field markup drives both sides.
pub fn generate_server(ast: &syn::DeriveInput, debug: &mut bool) -> Result<TokenStream, syn::Error> {
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

    let http_ctx = crate::consts::get_http_context();

    let http_fail_result = crate::consts::get_http_fail_result();

    let parse_http_input = match super::model_reader::generate(&struct_name, &input_fields) {
        Ok(result) => result,
        Err(err) => err.to_compile_error(),
    };

    let result = quote::quote! {
        impl #struct_name{
            pub async fn parse_http_input(http_route: &my_http_server::controllers::HttpRoute, ctx: &mut #http_ctx)->Result<Self,#http_fail_result>{
                use my_http_server::*;
                #print_request_to_console
                #parse_http_input
            }
        }
    };
    Ok(result.into())
}
