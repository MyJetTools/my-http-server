use std::str::FromStr;

use proc_macro2::TokenStream;

pub fn generate_handle_request_fn(input_data: Option<&str>) -> TokenStream {
    if let Some(input_data) = input_data {
        let input_data = TokenStream::from_str(input_data).unwrap();
        quote::quote! {
            // Model parsing now lives in my-http-utils (`parse` over the transport-free
            // `THttpRequest`). The body reaches the model one of two mutually exclusive ways, and
            // the model's own consts say which: materialized whole (READS_BODY) or taken as a
            // stream of chunks (STREAMS_BODY). Never both.

            // Taken first: it needs `&mut ctx.request`, and the reader below borrows it shared.
            let __body_stream = if #input_data::STREAMS_BODY {
                Some(ctx.request.take_body_stream()?)
            } else {
                None
            };

            let __body_bytes: Vec<u8> = if #input_data::READS_BODY {
                ctx.request.get_body().await?.as_slice().to_vec()
            } else {
                Vec::new()
            };

            // Scoped on purpose: the reader holds a `Cell` and is therefore not `Sync`, so it must
            // be gone before the `.await` below — otherwise the action's future stops being `Send`
            // and no longer satisfies `HandleHttpRequest`.
            let input_data = {
                let __reader = my_http_server::controllers::RequestReader::new(
                    &ctx.request,
                    http_route,
                    &__body_bytes,
                )
                .with_body_stream(__body_stream);

                #input_data::parse(&__reader)?
            };

            handle_request(self, input_data, ctx).await
        }
    } else {
        quote::quote!(handle_request(self, ctx).await)
    }
}
