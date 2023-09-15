/*
pub fn write_enum_type(yaml_writer: &mut YamlWriter, enum_structure: &HttpEnumStructure) {
    super::super::http_enum_type::build(yaml_writer, enum_structure);
}

pub fn write_enum_type(yaml_writer: &mut YamlWriter, enum_data: &HttpEnumStructure) {
    yaml_writer.increase_level();
    match &enum_data.enum_type {
        crate::controllers::documentation::EnumType::Integer => {
            yaml_writer.write("type", "integer");
            yaml_writer.write_array("enum", enum_data.cases.iter().map(|itm| itm.value.into()));
        }
        crate::controllers::documentation::EnumType::String => {
            yaml_writer.write("type", "string");
            yaml_writer.write_array(
                "enum",
                enum_data.cases.iter().map(|itm| itm.id.to_string().into()),
            );
        }
    }

    yaml_writer.decrease_level();
}
*/
