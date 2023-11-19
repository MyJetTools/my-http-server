use std::collections::HashMap;

use my_http_server::macros::*;
use my_http_server::types::FileContent;

use serde::Deserialize;

#[derive(MyHttpIntegerEnum)]
pub enum TestEnumFormData {
    #[http_enum_case(id = 0; description = "Case 1")]
    Case1,
    #[http_enum_case(id = 1; description = "Case 2")]
    Case2,
}

#[derive(MyHttpInput)]
pub struct FromDataInputModel {
    #[http_query(name = "dd"; description = "First param")]
    pub dd: String,
    #[http_query(description = "Second Param")]
    pub password: String,

    #[http_form_data(name = "strParam"; description = "String parameter")]
    pub str_param: String,

    #[http_form_data(name = "objParam"; description = "Object Param")]
    pub obj: BodyModelHttpInput,

    #[http_form_data(name = "hashmapOfString"; description = "Meta")]
    pub hashmap_of_string: HashMap<String, String>,

    #[http_form_data(name = "hashmapOfArrayOfString"; description = "Test Hashmap of array of string")]
    pub hashmap_of_array_of_string: HashMap<String, Vec<String>>,

    #[http_form_data(name = "hashmapOfArray"; description = "Meta")]
    pub hashmap_of_array: HashMap<String, Vec<BodyModelHttpInput>>,

    #[http_form_data(name = "subObject"; description = "Meta")]
    pub metaw: BodyModelHttpInput,

    #[http_form_data(name = "vecOfString"; description = "Meta")]
    pub vec_of_string: Vec<String>,

    #[http_form_data(name = "file"; description = "Meta")]
    pub file: FileContent,
}

#[derive(MyHttpInputObjectStructure, Debug, Deserialize)]
pub struct BodyModelHttpInput {
    pub test_field: String,
    pub test_field2: i64,
}
