use my_http_server::macros::{MyHttpInput, MyHttpStringEnum};

#[derive(MyHttpStringEnum)]
pub enum MyEnum {
    #[http_enum_case(id: "0",  description = "My Case 1 Description", default)]
    Case1,
    #[http_enum_case(id: "1",  description = "My Case 1 Description")]
    Case2,
    #[http_enum_case(id: "2", description = "My Case 1 Description")]
    Case3,
}

#[derive(MyHttpInput)]
pub struct TestInputModel {
    #[http_query(name = "myEnum", description = "a", default)]
    pub my_enum: MyEnum,
}
