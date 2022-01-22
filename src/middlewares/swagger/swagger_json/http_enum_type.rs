use crate::middlewares::{
    controllers::documentation::data_types::{EnumType, HttpEnumStructure},
    swagger::json_object_writer::JsonObjectWriter,
};

pub fn build(enum_structure: &HttpEnumStructure) -> JsonObjectWriter {
    let mut result = JsonObjectWriter::as_object();

    match enum_structure.enum_type {
        EnumType::Integer => {
            result.write_string_value("type", "integer");
        }
        EnumType::String => {
            result.write_string_value("type", "string");
        }
    }

    result.write_string_value("description", compile_description(enum_structure).as_str());

    result.write_object("enum", compile_enum(enum_structure));
    result.write_object("x-enumNames", compile_enum_names(enum_structure));

    result
}

fn compile_description(enum_structure: &HttpEnumStructure) -> String {
    let mut result = String::new();

    let mut first = true;

    for case in &enum_structure.cases {
        result.push_str(format!("{} = {}", case.id, case.description).as_str());

        if first {
            first = false;
        } else {
            result.push_str("\n");
        }
    }

    result
}

fn compile_enum(enum_structure: &HttpEnumStructure) -> JsonObjectWriter {
    let mut result = JsonObjectWriter::as_array();

    for case in &enum_structure.cases {
        result.write_number_element(case.id.to_string());
    }

    result
}

fn compile_enum_names(enum_structure: &HttpEnumStructure) -> JsonObjectWriter {
    let mut result = JsonObjectWriter::as_array();

    for case in &enum_structure.cases {
        result.write_string_element(case.value.as_str());
    }

    result
}
