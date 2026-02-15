use std::str::FromStr;

use proc_macro::TokenStream;
use types_reader::{macros::{MacrosParameters, MacrosEnum}, TokensObject};

use super::attributes::{ApiData, HttpResult, HttpResultModel};


#[derive(MacrosEnum)]
pub enum ActionMethod{
    #[value("GET")]
    Get,
    #[value("POST")]
    Post,
    #[value("PUT")]
    Put,
    #[value("DELETE")]
    Delete,
    #[value("OPTIONS")]
    Options
}

impl ActionMethod{
   pub fn get_trait_name(&self) -> proc_macro2::TokenStream {
    match self {
        ActionMethod::Get => {
            quote::quote!(my_http_server::controllers::actions::GetAction)
        }
        ActionMethod::Post => {
            quote::quote!(my_http_server::controllers::actions::PostAction)
        }
        ActionMethod::Put => {
            quote::quote!(my_http_server::controllers::actions::PutAction)
        }
        ActionMethod::Delete => {
            quote::quote!(my_http_server::controllers::actions::DeleteAction)
        }
        ActionMethod::Options => {
            quote::quote!(my_http_server::controllers::actions::OptionsAction)
        }
    }
  }
}


#[derive(MacrosParameters)]
pub struct HttpActionResult<'s>{
    pub status_code: u16,
    pub description: &'s str,
    #[allow_ident]
    pub model: Option<&'s str>,
}



#[derive(MacrosEnum)]
pub enum ShouldBeAuthorized{
    Yes,
    No,
    YesWithClaims(Vec<String>),
}

#[derive(MacrosParameters)]
pub struct ActionParameters<'s>{
    #[allow_ident]
    pub method: ActionMethod,
    pub route: &'s str,
    pub deprecated_routes: Option<Vec<&'s str>>,
    pub summary: Option<&'s str>,
    pub description: Option<&'s str>,
    pub controller: Option<&'s str>,
    #[allow_ident]
    pub input_data: Option<&'s str>,
    pub authorized: Option<ShouldBeAuthorized>,
    pub result: Option<Vec<HttpActionResult<'s>>>,
}



impl<'s> ActionParameters<'s>{

    pub fn get_should_be_authorized(&self) -> Result<proc_macro2::TokenStream, syn::Error> {
        if self.authorized.is_none() {
            return Ok(quote::quote!(ShouldBeAuthorized::UseGlobal));
        }

        let should_be_authorized = self.authorized.as_ref().unwrap();


        match should_be_authorized{
            ShouldBeAuthorized::Yes => {
                 Ok(quote::quote!(ShouldBeAuthorized::Yes))
            },
            ShouldBeAuthorized::No => {
                Ok(quote::quote!(ShouldBeAuthorized::No))
            },
            ShouldBeAuthorized::YesWithClaims(claims) => {
                if claims.len() == 0 {
                    return Ok(quote::quote!(ShouldBeAuthorized::Yes));
                }
    
                let mut result = Vec::new();
    
                for itm in claims {
                    result.push(quote::quote!(#itm));
                }
    
                return Ok(quote::quote!(ShouldBeAuthorized::YesWithClaims(
                    my_http_server::controllers::RequiredClaims::from_vec(
                        vec![#(#result.to_string(),)*]
                    )
                ))
                .into());
            },
        }

    }
    
    pub fn get_api_data(&self) -> Option<ApiData<'s>>{
        if self.controller.is_none(){
            return None;
        }

        let controller = self.controller.unwrap();


        if self.summary.is_none(){
             panic!("'summary' field is required");
        }
        

        let summary = self.summary.unwrap();


        if self.description.is_none(){
            panic!("'description' field is required");
       }
       

        let description = self.description.unwrap();



        let mut results = None;



        if let Some(src_results) =  &self.result{
            let mut out_results = Vec::with_capacity(src_results.len());
            for item in src_results{
                out_results.push(HttpResult{
                    status_code: item.status_code,
                    description: item.description.to_string(),
                    result_type: if let Some(result_type) = item.model{
                        Some(HttpResultModel::create(result_type))
                    }else{
                        None
                    },
                });
            }

            results = Some(out_results);
        }


        Some(ApiData{
            controller,
            summary,
            description,
            results 
        })
    }
}


pub fn build_action(attr: TokenStream, input: TokenStream) -> Result<TokenStream, syn::Error> {

    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    let attr: proc_macro2::TokenStream = attr.into();
    let tokens_object: TokensObject = attr.try_into()?;
    let action_parameters:ActionParameters = (&tokens_object).try_into()?;

    let struct_name = &ast.ident;

    let trait_name = action_parameters.method.get_trait_name();

    let route = action_parameters.route;

    let http_action_description = crate::consts::get_http_action_description_with_ns();

    let description = super::generate_http_action_description_fn(&action_parameters)?;

    let http_route = crate::consts::get_http_route();

    let http_context = crate::consts::get_http_context();

    let http_ok_result = crate::consts::get_http_ok_result();

    let http_fail_result = crate::consts::get_http_fail_result();

    let handle_request = super::generate_handle_request_fn(action_parameters.input_data);

    let model_routes: proc_macro2::TokenStream = if let Some(input_data) = &action_parameters.input_data{
        let input_data = proc_macro2::TokenStream::from_str(input_data).unwrap();
        quote::quote!(#input_data::get_model_routes())    
    }else{
        quote::quote!(None)
    };


    let deprecated_routes = if let Some(deprecated_routes) = action_parameters.deprecated_routes{

        if deprecated_routes.is_empty(){
            panic!("'deprecated_routes' must have at least one route");
        }

        quote::quote!(Some(vec![#(#deprecated_routes,)*]))

    }else{
        quote::quote!(None)
    };

    let result = quote::quote! {
        #[derive(Clone)]
        #ast

        impl #struct_name{
            fn get_description() -> Option<#http_action_description>{
                #description
            }
        }

        impl #trait_name for #struct_name{
            fn get_route(&self) -> &'static str {
                #route               
            }

            fn get_deprecated_routes(&self) -> Option<Vec<&'static str>>{
                #deprecated_routes
            }

            fn get_model_routes(&self) -> Option<Vec<&'static str>>{
                #model_routes
            }
        }

        impl my_http_server::controllers::actions::GetDescription for #struct_name{
            fn get_description(&self) -> Option<#http_action_description>{
                Self::get_description()
            }
        }

        #[async_trait::async_trait]
        impl my_http_server::controllers::actions::HandleHttpRequest for #struct_name{
            async fn handle_request(&self, http_route: &#http_route, ctx: &mut #http_context) -> Result<#http_ok_result, #http_fail_result> {
                
                #handle_request
            }
        }
  
    }
    .into();

   Ok(result)
}
