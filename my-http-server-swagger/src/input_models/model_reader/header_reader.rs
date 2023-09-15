use crate::{input_models::input_fields::InputFields, reflection::PropertyType};

pub fn init_header_variables(result: &mut String, input_fields: &InputFields) {
    for input_field_header in input_fields.get_from_header_elements() {
        let mut valid_type = false;

        if input_field_header.property.ty.is_string() {
            valid_type = true;
        }

        if let PropertyType::OptionOf(inner_generic) = &input_field_header.property.ty {
            if inner_generic.is_string() {
                valid_type = true;
            }
        }

        if !valid_type {
            panic!(
                "Can not read {} type to from header to property {}",
                input_field_header.property.ty.as_str(),
                input_field_header.struct_field_name()
            );
        }

        let line = if input_field_header.required() {
            format!("let {field_name} = ctx.request.get_required_header(\"{header_key}\")?.to_string();\n", field_name=input_field_header.struct_field_name(), header_key=input_field_header.name())
        } else {
            let reading_command = format!(
                "ctx.request.get_optional_header(\"{header_key}\")",
                header_key = input_field_header.name()
            );

            let reading_command = option_of_str_to_option_of_string(reading_command.as_str());

            format!(
                "let {field_name} = {reading_command};\n",
                field_name = input_field_header.struct_field_name(),
            )
        };

        result.push_str(line.as_str());
    }
}

fn option_of_str_to_option_of_string(expr: &str) -> String {
    format!(
        r###"
        if let Some(value) = {expr}{{
            Some(value.to_string())
        }}else{{
            None
        }};
    "###,
        expr = expr,
    )
}
