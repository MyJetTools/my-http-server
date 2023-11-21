use my_http_server::macros::*;
use my_http_server::*;

#[derive(MyHttpStringEnum)]
pub enum MyEnumModel {
    #[http_enum_case(id: 0, value:"myCase1", description = "My Case 1 Description", default)]
    Case1,
    #[http_enum_case(id: 1, description = "My Case 1 Description")]
    Case2,
    #[http_enum_case(id: 2, description = "My Case 1 Description")]
    Case3,
}

#[derive(MyHttpInput)]
pub struct MyHttpInput {
    #[http_query(name = "myEnum", description = "a", default)]
    pub my_enum: MyEnumModel,
}

#[cfg(test)]
mod tests {
    use super::*;

    impl MyEnumModel {
        pub fn assert_case_number(&self, number: i32) {
            match self {
                MyEnumModel::Case1 => assert_eq!(number, 1),
                MyEnumModel::Case2 => assert_eq!(number, 2),
                MyEnumModel::Case3 => assert_eq!(number, 2),
            }
        }
    }
    #[test]
    fn test_overridden_value() {
        let default = MyEnumModel::create_default().unwrap();

        default.assert_case_number(1);

        assert_eq!("myCase1", MyEnumModel::default_as_str());

        assert_eq!("myCase1", MyEnumModel::default_as_str());
    }
}
