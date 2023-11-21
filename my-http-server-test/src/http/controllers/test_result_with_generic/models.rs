use my_http_server::{controllers::documentation::DataTypeProvider, macros::*};

use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize_repr, Deserialize_repr, MyHttpIntegerEnum, Debug)]
#[repr(i16)]
pub enum Status {
    #[http_enum_case(id="0"; description="Operations was successful")]
    Ok,
    #[http_enum_case(id="1"; description="Operations was not successful")]
    NotOk,
}

#[derive(MyHttpObjectStructure)]
pub struct ResponseWithResult<TData: DataTypeProvider> {
    pub status: Status,
    pub data: TData,
}

#[derive(MyHttpObjectStructure)]
pub struct MyData {
    pub id: i32,
    pub name: String,
}

#[derive(MyHttpObjectStructure)]
pub struct MyData2 {
    pub id2: i32,
    pub name2: String,
}
