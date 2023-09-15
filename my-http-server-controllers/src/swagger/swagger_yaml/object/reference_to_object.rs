use crate::{
    controllers::documentation::InputStructure, swagger::swagger_yaml::yaml_writer::YamlWriter,
};

pub fn write_reference_to_object(yaml_writer: &mut YamlWriter, simple_type: &impl InputStructure) {
    yaml_writer.write(
        "$ref",
        format!(
            "'#/components/schemas/{}'",
            simple_type.get_struct_id().as_str()
        )
        .as_str(),
    );
}
