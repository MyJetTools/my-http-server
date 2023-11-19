use std::collections::HashMap;

use my_http_server::macros::*;
use serde::{Deserialize, Serialize};

#[derive(MyHttpInput)]
pub struct BodyModelHttpInput {
    #[http_body_raw(name = "body"; description = "Body")]
    pub body: BodyModelContract,
}

#[derive(Deserialize, Debug, MyHttpInputObjectStructure)]
pub struct BodyModelContract {
    pub field1: usize,
    pub field2: String,
    pub field3: BodySubModel,
}

#[derive(Deserialize, Debug, MyHttpInputObjectStructure)]
pub struct BodySubModel {
    pub field1: usize,
    pub field2: String,
}

#[derive(MyHttpInput)]
pub struct BodyModelStringRawHttpInput {
    #[http_body_raw(name = "body"; description = "Body")]
    pub body: String,
}

#[derive(MyHttpInput)]
pub struct BodyModelU8RawHttpInput {
    #[http_body_raw(name = "body"; description = "Body")]
    pub body: u8,
}

#[derive(MyHttpInput)]
pub struct BodyModelI32RawHttpInput {
    #[http_body_raw(name = "body"; description = "Body")]
    pub body: i32,
}

#[derive(MyHttpInput)]
pub struct BodyModelVecOfI32RawHttpInput {
    #[http_body_raw(name = "body"; description = "Body")]
    pub body: Vec<i32>,
}

#[derive(MyHttpInput)]
pub struct BodyModelVecOfStringRawHttpInput {
    #[http_body_raw(name = "body"; description = "Body")]
    pub body: Vec<String>,
}

#[derive(MyHttpInput)]
pub struct BodyModelVecOfObjectRawHttpInput {
    #[http_body_raw(name = "body"; description = "Body")]
    pub body: Vec<BodyModelContract>,
}

#[derive(MyHttpInput)]
pub struct BodyModelHashmapOfObjectRawHttpInput {
    #[http_body_raw(name = "body"; description = "Body")]
    pub body: HashMap<String, BodyModelContract>,
}

#[derive(Debug, MyHttpInput)]

pub struct BodyAsModel {
    #[http_body(name: "tradingPackageId", description:"Trading Package Id")]
    pub trading_package_id: String,
    #[http_body(name: "tradingPlatform", description:"Trading platform")]
    pub trading_platform: TradingPlatformModel,
}

#[derive(Debug, MyHttpIntegerEnum, Default, Serialize)]
pub enum TradingPlatformModel {
    #[default]
    #[http_enum_case(id: 0; description:"MetaTrader4")]
    MetaTrader4,
    #[http_enum_case(id: 1; description:"MetaTrader5")]
    MetaTrader5,
}

#[derive(Debug, MyHttpIntegerEnum)]
pub enum BrokerModel {
    #[http_enum_case(id: 0; description:"Welltrade")]
    Welltrade,
}
