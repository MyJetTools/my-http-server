use proc_macro::TokenStream;
use types_reader::EnumCase;

pub fn generate(ast: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    let struct_name = &ast.ident;
    let src_fields = EnumCase::read(ast)?;
    todo!("Implement");
}
