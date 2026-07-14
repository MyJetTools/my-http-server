use std::str::FromStr;

use proc_macro2::TokenStream;

pub fn generate_handle_request_fn(input_data: Option<&str>) -> TokenStream {
    if let Some(input_data) = input_data {
        let input_data = TokenStream::from_str(input_data).unwrap();
        quote::quote! {
            // Model parsing now lives in my-http-utils (`parse` over the transport-free
            // `THttpRequest`). The body is received here only when the model reads it, then handed
            // to the reader as an already-materialized slice so `parse` stays synchronous.
            let __body_bytes: Vec<u8> = if #input_data::READS_BODY {
                ctx.request.get_body().await?.as_slice().to_vec()
            } else {
                Vec::new()
            };
            let __reader = my_http_server::controllers::RequestReader::new(
                &ctx.request,
                http_route,
                &__body_bytes,
            );
            let input_data = #input_data::parse(&__reader)?;
            handle_request(self, input_data, ctx).await
        }
    } else {
        quote::quote!(handle_request(self, ctx).await)
    }
}
