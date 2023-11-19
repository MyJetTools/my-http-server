#![allow(unused)]
use std::collections::HashMap;

use my_http_server::macros::*;
use serde::{Deserialize, Serialize};

#[derive(MyHttpIntegerEnum, Debug)]
pub enum TestEnum {
    #[http_enum_case(id: 0, description: "Case 1")]
    Case1,
    #[http_enum_case(id: 1; description: "Case 2")]
    Case2,
}

#[derive(MyHttpInput, Debug)]
pub struct PostBodyModel {
    #[http_query(name = "dd"; description = "First param", to_lower_string)]
    pub dd: String,

    #[http_query(description = "Second Param")]
    pub password: String,

    #[http_query(name = "ddOpt"; description = "First param")]
    pub dd_opt: Option<String>,

    #[http_query(name="passwordOpt" description = "Second Param")]
    pub password_opt: String,

    #[http_query(name = "queryArray", description = "Query array param")]
    pub query_array: Vec<String>,

    #[http_query(name = "enumField", description = "Test enum field")]
    pub enum_field: TestEnum,

    #[http_body(name = "dtFromOpt"; description = "Third param", default:"MyDefault")]
    pub dt_from_opt: String,

    #[http_body(name = "dtToOpt"; description = "DateTime")]
    pub dt_to_opt: String,

    #[http_body(name = "enumFieldBody", description = "Test enum field as body")]
    pub enum_field_body: TestEnum,

    #[http_body(name = "hashmapOfString"; description = "Meta")]
    pub hashmap_of_string: HashMap<String, String>,

    #[http_body(name = "hashmapOfArrayOfString"; description = "Test Hashmap of array of string")]
    pub hashmap_of_array_of_string: HashMap<String, Vec<String>>,

    #[http_body(name = "hashmapOfArray"; description = "Meta")]
    pub hashmap_of_array: HashMap<String, Vec<DoneRequest>>,

    #[http_body(name = "subObject"; description = "Meta")]
    pub metaw: DoneRequest,

    #[http_body(name = "vecOfString"; description = "Meta")]
    pub vec_of_string: Vec<String>,
}

#[derive(Deserialize, Debug, MyHttpInputObjectStructure)]
pub struct DoneRequest {
    #[serde(rename = "doneResult")]
    result: u8,
}

#[derive(Serialize, Debug, MyHttpObjectStructure)]
pub struct GetKeyValueResponse {
    pub value: Option<String>,
}

#[derive(Serialize, Debug, MyHttpObjectStructure)]
pub struct DoneResult {
    #[serde(rename = "doneResult")]
    //#[debug]
    result: u8,
}
