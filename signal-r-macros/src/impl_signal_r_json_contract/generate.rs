use types_reader::{macros::MacrosParameters, TokensObject};

#[derive(MacrosParameters)]
pub struct MacrosParameters {
    #[default]
    pub action_name: String,
}

pub fn generate(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> Result<proc_macro::TokenStream, syn::Error> {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    let attr: proc_macro2::TokenStream = attr.into();
    let attrs = TokensObject::new(attr.into())?;
    let parameters: MacrosParameters = (&attrs).try_into()?;

    let action_name = parameters.action_name.as_str();

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

            fn deserialize<'s>(src: impl Iterator<Item = &'s [u8]>) -> Result<Self::Item, String> {
                for payload in src {
                    let result = serde_json::from_slice(payload);
                    if let Err(err) = &result {
                        return Err(format!(
                            "Invalid message during deserialization for action: {}. Error: {}",
                            Self::ACTION_NAME,
                            err
                        ));
                    }
                    let result: Self = result.unwrap();
                    return Ok(result);
                }

                return Err(format!(
                    "Can not be 0 parameters amount during deserialization for action: {}",
                    Self::ACTION_NAME
                ));
            }
        }
    };

    Ok(result.into())
}
