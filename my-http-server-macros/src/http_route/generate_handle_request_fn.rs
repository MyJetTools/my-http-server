use std::str::FromStr;

use proc_macro2::TokenStream;

pub fn generate_handle_request_fn(input_data: Option<&str>) -> TokenStream {
    if let Some(input_data) = input_data {
        let input_data = TokenStream::from_str(input_data).unwrap();
        quote::quote! {
            // Model parsing now lives in my-http-utils (`parse` / `parse_with_body_stream` over
            // the transport-free `THttpRequest`). The model's own consts say how the body reaches
            // it:
            // * named body fields (READS_BODY without READS_BODY_RAW) — read off the wire as the
            //   body arrives, keeping only the fields the model names;
            // * `#[http_body_raw]` (READS_BODY_RAW) — materialized whole;
            // * `#[http_body_as_stream]` (STREAMS_BODY) — taken as a stream of chunks.

            // `None` for a body that has to be materialized after all: one a middleware already
            // materialized, or one that announced a `Content-Encoding` — only that path decodes.
            let __incoming_body = if #input_data::READS_BODY && !#input_data::READS_BODY_RAW {
                ctx.request.take_incoming_body_stream()
            } else {
                None
            };

            let input_data = if let Some(__incoming_body) = __incoming_body {
                // Shared: the parse takes the stream, and the rest is drained after it
                let __incoming_body = std::sync::Arc::new(__incoming_body);

                let __reader = my_http_server::controllers::RequestReader::new(
                    &ctx.request,
                    http_route,
                    &[],
                );

                let input_data =
                    #input_data::parse_with_body_stream(&__reader, __incoming_body.clone()).await?;

                // The parse stops reading once it has the fields the model reads. The rest of
                // the body is read and let go before the action runs: a body cut short must not
                // reach it, and the connection is then fit to be used again.
                __incoming_body.drain().await?;

                input_data
            } else {
                // Taken first: it needs `&mut ctx.request`, and the reader below borrows it
                // shared.
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
