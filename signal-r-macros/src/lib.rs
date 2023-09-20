mod impl_signal_r_json_contract;

#[proc_macro_attribute]
pub fn signal_r_json_contract(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match impl_signal_r_json_contract::generate(attr, input) {
        Ok(result) => result,
        Err(err) => err.into_compile_error().into(),
    }
}
