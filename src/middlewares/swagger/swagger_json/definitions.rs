use crate::middlewares::{
    controllers::ControllersMiddleware, swagger::json_object_writer::JsonObjectWriter,
};

pub fn build(controllers: &ControllersMiddleware) -> Option<JsonObjectWriter> {
    if controllers.http_objects.len() == 0 {
        return None;
    }

    let mut result = JsonObjectWriter::as_object();

    for http_object in &controllers.http_objects {
        result.write_object(
            &http_object.struct_id,
            super::http_object_type::build(http_object),
        );
    }

    Some(result)
}
