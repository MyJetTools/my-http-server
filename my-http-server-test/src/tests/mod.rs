use my_http_server::macros::MyHttpInput;

mod test_defaults;
#[cfg(test)]
mod test_string_enum_defaults;
mod test_to_lower_case;

//#[derive(MyHttpInput)]
pub struct SetBalanceLockApiRequest {
    //   #[http_path(name = "id"; description = "Id of wallet")]
    pub wallet_id: String,
    //   #[http_form_data(name = "IsLocked"; description = "Is locked")]
    pub is_locked: bool,
    //   #[http_form_data(name = "BalanceId"; description = "Id of balance")]
    pub balance_id: String,
}

impl SetBalanceLockApiRequest {
    pub fn get_input_params(
    ) -> Vec<my_http_server::controllers::documentation::in_parameters::HttpInputParameter> {
        use my_http_server::controllers::documentation::*;
        vec![
            in_parameters::HttpInputParameter {
                field: data_types::HttpField::new("IsLocked", bool::get_data_type(), true),
                description: "Is locked".to_string(),
                source: in_parameters::HttpParameterInputSource::FormData,
            },
            in_parameters::HttpInputParameter {
                field: data_types::HttpField::new("BalanceId", String::get_data_type(), true),
                description: "Id of balance".to_string(),
                source: in_parameters::HttpParameterInputSource::FormData,
            },
            in_parameters::HttpInputParameter {
                field: data_types::HttpField::new("id", String::get_data_type(), true),
                description: "Id of wallet".to_string(),
                source: in_parameters::HttpParameterInputSource::Path,
            },
        ]
    }
    pub async fn parse_http_input(
        http_route: &my_http_server::controllers::HttpRoute,
        ctx: &mut my_http_server::HttpContext,
    ) -> Result<Self, my_http_server::HttpFailResult> {
        use my_http_server::*;
        let wallet_id: String = http_route
            .get_value(&ctx.request.http_path, "id")?
            .try_into()?;
        let (is_locked, balance_id) = {
            let __body = ctx.request.get_body().await?;
            let __reader = __body.get_form_data_reader()?;
            let is_locked = __reader.get_required("IsLocked")?.try_into()?;
            let balance_id: String = __reader.get_required("BalanceId")?.try_into()?;
            (is_locked, balance_id)
        };
        Ok(SetBalanceLockApiRequest {
            wallet_id,
            is_locked,
            balance_id,
        })
    }
    pub fn get_model_routes() -> Option<Vec<&'static str>> {
        Some(vec!["id"])
    }
}
