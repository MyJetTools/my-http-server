use types_reader::ParamsList;

pub fn generate(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> Result<proc_macro::TokenStream, syn::Error> {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    let attrs = ParamsList::new(attr.into(), || None)?;

    let action_name = attrs.get_from_single_or_named("action_name")?;
    let action_name = action_name.unwrap_as_string_value()?;
    let action_name = action_name.as_str();

    let struct_name = &ast.ident;

    let result = quote::quote! {
        #ast

        impl my_http_server::signal_r::SignalRContractSerializer for  #struct_name {
            const ACTION_NAME: &'static str = #action_name;
            type Item = #struct_name;

            fn serialize(&self) -> Vec<Vec<u8>> {
                let json = serde_json::to_vec(&self);
                return vec![json.unwrap()];
            }

            fn deserialize(src: &[&[u8]]) -> Result<Self::Item, String> {
                if src.len() != 1 {
                    return Err(format!(
                        "Invalid messages amount {} during deserialization for action: {}",
                        src.len(),
                        Self::ACTION_NAME
                    ));
                }

                let payload = src.get(0).unwrap();

                let result = serde_json::from_slice(payload);

                if let Err(err) = &result {
                    return Err(format!(
                        "Invalid message during deserialization for action: {}. Error: {}",
                        Self::ACTION_NAME,
                        err
                    ));
                }

                result.unwrap()
            }
        }
    };

    Ok(result.into())
}
