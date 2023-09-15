use crate::controllers::ControllersAuthorization;

use super::yaml_writer::YamlWriter;

pub fn build(yaml_writer: &mut YamlWriter, auth: &Option<ControllersAuthorization>) {
    if auth.is_none() {
        return;
    }

    let auth = auth.as_ref().unwrap();

    yaml_writer.write_upper_level("securitySchemes", |yaml_writer| match auth {
        ControllersAuthorization::BasicAuthentication {
            global: _,
            global_claims: _,
        } => {
            yaml_writer.write_upper_level("BasicAuth", |yaml_writer| {
                yaml_writer.write("type", "http");
                yaml_writer.write("scheme", "basic");
            });
        }
        ControllersAuthorization::ApiKeys {
            global: _,
            global_claims: _,
        } => {
            yaml_writer.write_upper_level("ApiKeyAuth", |yaml_writer| {
                yaml_writer.write("type", "apiKey");
                yaml_writer.write("in", "header");
                yaml_writer.write("name", "X-API-Key");
            });
        }
        ControllersAuthorization::BearerAuthentication {
            global: _,
            global_claims: _,
        } => {
            yaml_writer.write_upper_level("BearerAuth", |yaml_writer| {
                yaml_writer.write("type", "http");
                yaml_writer.write("scheme", "bearer");
            });
        }
    });
}
