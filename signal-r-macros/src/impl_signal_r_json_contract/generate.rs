use types_reader::ParamsList;

pub fn generate(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> Result<proc_macro::TokenStream, syn::Error> {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    let attrs = ParamsList::new(attr.into(), || None)?;

    let action_name = attrs.get_from_single_or_named("action_name")?;

    let struct_name = &ast.ident;

    let result = quote::quote! {
        #ast

        impl my_http_server::signal_r::SignalRContractSerializer for  #struct_name {
            const ACTION_NAME: &'static str = action_name;

            fn serialize(self) -> Vec<Vec<u8>> {
                let json = serde_json::to_vec(&self);
                return vec![json.unwrap()];
            }

            fn deserialize<'s>(src: &'s [Vec<u8>]) -> Self {
                if src.len() != 1 {
                    panic!(
                        "Invalid messages amount during deserialization for action: {}",
                        Self::ACTION_NAME
                    );
                }

                let payload = src.get(0).unwrap();

                let result = serde_json::from_slice(payload);

                if let Err(err) = &result {
                    panic!(
                        "Invalid message during deserialization for action: {}. Error: {}",
                        Self::ACTION_NAME,
                        err
                    );
                }

                result.unwrap()
            }
        }
    };

    Ok(result.into())
}
