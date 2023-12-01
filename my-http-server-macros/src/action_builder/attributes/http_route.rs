use types_reader::TokensObject;

use super::{http_method::HttpMethod, ApiData};

pub struct HttpRouteModel<'s> {
    pub method: HttpMethod,
    pub route: &'s str,
    pub input_data: Option<&'s str>,
    pub api_data: Option<ApiData<'s>>,
    pub should_be_authorized: Option<&'s TokensObject>,
}

impl<'s> HttpRouteModel<'s> {
    pub fn parse(attrs: &'s types_reader::TokensObject) -> Result<Self, syn::Error> {
        let method = attrs
            .get_named_param("method")?
            .get_value()?
            .get_any_value_as_str()?;

        let route = attrs
            .get_named_param("route")?
            .get_value()?
            .as_string()?
            .as_str();

        let input_data = if let Some(input_data) = attrs.try_get_named_param("input_data") {
            Some(input_data.get_value()?.get_any_value_as_str()?)
        } else {
            None
        };

        let should_be_authorized = attrs.try_get_named_param("authorized");

        let result = if let Some(controller) = attrs.try_get_named_param("controller") {
            let controller = controller.get_value()?.as_string()?.as_str();

            Ok(Self {
                method: HttpMethod::parse(method),
                route,
                input_data,
                should_be_authorized,
                api_data: Some(ApiData::new(controller, attrs)?),
            })
        } else {
            Ok(Self {
                method: HttpMethod::parse(method),
                route,
                input_data,
                should_be_authorized,
                api_data: None,
            })
        };

        result
    }

    pub fn get_should_be_authorized(&self) -> Result<proc_macro2::TokenStream, syn::Error> {
        if self.should_be_authorized.is_none() {
            return Ok(quote::quote!(ShouldBeAuthorized::UseGlobal));
        }

        let should_be_authorized = self.should_be_authorized.unwrap();

        if let Some(string_value) = should_be_authorized.get_value()?.try_as_string() {
            let value = string_value.as_str();

            if value == "Yes" || value == "[]" {
                return Ok(quote::quote!(ShouldBeAuthorized::Yes));
            }

            if value == "No" {
                return Ok(quote::quote!(ShouldBeAuthorized::No));
            }

            return Err(should_be_authorized
                .throw_error("Unsupported value. It should be Yes, No or Array of strings"));
        }

        if let Some(values) = should_be_authorized.try_get_vec() {
            if values.len() == 0 {
                return Ok(quote::quote!(ShouldBeAuthorized::Yes));
            }

            let mut result = Vec::new();

            for itm in values {
                let itm = itm.get_value()?.as_string()?.as_str();
                result.push(quote::quote!(#itm));
            }

            return Ok(quote::quote!(ShouldBeAuthorized::YesWithClaims(
                my_http_server::controllers::RequiredClaims::from_vec(
                    vec![#(#result.to_string(),)*]
                )
            ))
            .into());
        }

        Err(should_be_authorized.throw_error("Unsupported data type"))
    }
}
