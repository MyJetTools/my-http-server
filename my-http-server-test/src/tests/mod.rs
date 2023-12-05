use my_http_server::macros::MyHttpInput;

mod test_http_input_password_field;

mod test_defaults;
mod test_http_input_email_field;
#[cfg(test)]
mod test_string_enum_defaults;
#[cfg(test)]
mod test_to_lower_case;

#[derive(MyHttpInput)]
pub struct SetBalanceLockApiRequest {
    #[http_path(name = "id"; description = "Id of wallet")]
    pub wallet_id: String,
    #[http_form_data(name = "IsLocked"; description = "Is locked")]
    pub is_locked: bool,
    #[http_form_data(name = "BalanceId"; description = "Id of balance")]
    pub balance_id: String,
}
