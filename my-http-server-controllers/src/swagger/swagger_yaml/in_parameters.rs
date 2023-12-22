use crate::controllers::documentation::HttpActionDescription;

use super::{in_param_as_body, in_param_as_from_data, yaml_writer::YamlWriter};

pub fn build(yaml_writer: &mut YamlWriter, action_description: &HttpActionDescription) {
    if let Some(non_body_params) = action_description.input_params.get_non_body_params() {
        yaml_writer.write_upper_level("parameters", |yaml_writer| {
            for param in non_body_params {
                yaml_writer.write_upper_level_with_value(
                    "- in",
                    param.source.as_str().into(),
                    |upper_level| {
                        super::query_params::write_query_input_param(upper_level, param);
                    },
                );
            }
        });
    }

    if let Some(body_param) = action_description.input_params.is_single_body_parameter() {
        yaml_writer.write_upper_level("requestBody", |yaml_writer| {
            yaml_writer.write_as_str("description", body_param.description.as_str());
            yaml_writer.write_bool("required", true);
            yaml_writer.write_upper_level("content", |yaml_writer| {
                yaml_writer.write_upper_level("application/json", |yaml_writer| {
                    super::http_data_type::build(
                        yaml_writer,
                        "schema",
                        &body_param.field.data_type,
                    );
                });
            });
        });

        return;
    }

    if let Some(body_params) = action_description.input_params.get_body_params() {
        yaml_writer.write_upper_level("requestBody", |yaml_writer| {
            yaml_writer.write_bool("required", true);
            yaml_writer.write_upper_level("content", |yaml_writer| {
                yaml_writer.write_upper_level("application/json", |yaml_writer| {
                    yaml_writer.write_upper_level("schema", |yaml_writer| {
                        yaml_writer.write("type", "object");
                        yaml_writer.write_upper_level("properties", |yaml_writer| {
                            for param in body_params {
                                in_param_as_body::write(yaml_writer, &param.field);
                            }
                        });
                    });
                });
            });
        });
    }

    if let Some(form_data_params) = action_description.input_params.get_form_data_params() {
        yaml_writer.write_upper_level("requestBody", |yaml_writer| {
            yaml_writer.write_upper_level("content", |yaml_writer| {
                let objects = yaml_writer.write_upper_level_with_ctx(
                    "multipart/form-data",
                    Vec::new(),
                    |ctx, yaml_writer| {
                        yaml_writer.write_upper_level_with_ctx("schema", ctx, |ctx, yaml_writer| {
                            yaml_writer.write("type", "object");
                            yaml_writer.write_upper_level_with_ctx(
                                "properties",
                                ctx,
                                |mut ctx, yaml_writer| {
                                    for param in form_data_params {
                                        in_param_as_from_data::write(yaml_writer, &param.field);
                                        if param.field.data_type.is_object() {
                                            ctx.push(param);
                                        }
                                    }

                                    ctx
                                },
                            )
                        })
                    },
                );

                if objects.len() > 0 {
                    yaml_writer.write_upper_level("encoding", |yaml_writer| {
                        for obj in &objects {
                            yaml_writer.write_upper_level(obj.field.name.as_str(), |yaml_writer| {
                                yaml_writer.write("contentType", "application/json");
                            });
                        }
                    });
                }
            });
        });
    }
}
