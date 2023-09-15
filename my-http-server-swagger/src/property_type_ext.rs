use types_reader::PropertyType;

pub trait PropertyTypeExt {
    fn is_file_content(&self) -> bool;
    fn required(&self) -> bool;
}

impl<'s> PropertyTypeExt for PropertyType<'s> {
    fn is_file_content(&self) -> bool {
        match self {
            PropertyType::Struct(name, _) => name == "FileContent",
            _ => false,
        }
    }

    fn required(&self) -> bool {
        match &self {
            PropertyType::VecOf(_) => false,
            PropertyType::OptionOf(_) => false,
            _ => true,
        }
    }
}
