use crate::{
    middlewares::{
        controllers::documentation::{
            data_types::HttpDataType, out_results::HttpResult, HttpActionDescription,
        },
        swagger::json_object_writer::JsonObjectWriter,
    },
    WebContentType,
};

pub fn build(action_description: &HttpActionDescription) -> JsonObjectWriter {
    let mut result = JsonObjectWriter::as_object();

    result.write_object("tags", compile_tags(action_description));
    result.write_string_value("description", action_description.description);
    result.write_object("produces", compile_produces(action_description));
    result.write_object("responses", compile_responses(&action_description.results));

    result.write_object(
        "parameters",
        super::in_parameters::build(&action_description),
    );

    result
}

fn compile_tags(action_description: &HttpActionDescription) -> JsonObjectWriter {
    let mut result = JsonObjectWriter::as_array();
    result.write_string_element(action_description.name);
    result
}

fn compile_produces(action_description: &HttpActionDescription) -> JsonObjectWriter {
    let mut produces = Vec::new();

    for http_result in &action_description.results {
        let produce_type = match http_result.data_type {
            HttpDataType::SimpleType(_) => Some(WebContentType::Text.as_str()),
            HttpDataType::Object { struct_id: _ } => Some(WebContentType::Json.as_str()),
            HttpDataType::None => None,
            HttpDataType::ArrayOf(_) => None,
        };

        if let Some(produce_type) = produce_type {
            if !produces.iter().any(|itm| itm == produce_type) {
                produces.push(produce_type.to_string());
            }
        }
    }

    let mut result = JsonObjectWriter::as_array();

    for produce in produces {
        result.write_string_element(produce.as_str())
    }

    result
}

fn compile_responses(results: &[HttpResult]) -> JsonObjectWriter {
    let mut result = JsonObjectWriter::as_object();

    for http_result in results {
        result.write_object(
            format!("{}", http_result.http_code).as_str(),
            compile_response(http_result),
        );
    }

    result
}

fn compile_response(src: &HttpResult) -> JsonObjectWriter {
    let mut result = JsonObjectWriter::as_object();
    result.write_bool_value("x-nullable", src.nullable);
    result.write_string_value("description", src.description.as_str());
    if let Some(obj) = super::http_data_type::build(&src.data_type) {
        result.write_object("schema", obj);
    }

    result
}
