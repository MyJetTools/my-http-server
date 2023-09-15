use proc_macro2::TokenStream;

pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}

impl HttpMethod {
    pub fn parse(src: &str) -> Self {
        match src {
            "GET" => HttpMethod::Get,
            "POST" => HttpMethod::Post,
            "PUT" => HttpMethod::Put,
            "DELETE" => HttpMethod::Delete,
            _ => panic!("Unsupported http method: {}", src),
        }
    }
    pub fn get_trait_name(&self) -> TokenStream {
        match self {
            HttpMethod::Get => {
                quote::quote!(my_http_server_controllers::controllers::actions::GetAction)
            }
            HttpMethod::Post => {
                quote::quote!(my_http_server_controllers::controllers::actions::PostAction)
            }
            HttpMethod::Put => {
                quote::quote!(my_http_server_controllers::controllers::actions::PutAction)
            }
            HttpMethod::Delete => {
                quote::quote!(my_http_server_controllers::controllers::actions::DeleteAction)
            }
        }
    }
}
