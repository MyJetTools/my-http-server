use std::collections::BTreeMap;

use crate::controllers::documentation::HttpActionDescription;

use crate::controllers::ControllersMiddleware;

use super::yaml_writer::YamlWriter;

pub fn build(
    yaml_writer: &mut YamlWriter,

    actions: &BTreeMap<String, BTreeMap<String, HttpActionDescription>>,
    controllers: &ControllersMiddleware,
) {
    yaml_writer.write_upper_level("paths", |yaml_writer| {
        for (path, actions) in actions {
            yaml_writer.write_upper_level(path, |yaml_writer| {
                for (verb, action_description) in actions {
                    super::verb_description::build(
                        yaml_writer,
                        verb,
                        action_description,
                        controllers,
                    )
                }
            });
        }
    });
}
