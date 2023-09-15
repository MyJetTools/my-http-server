use types_reader::ParamsList;

use super::HttpResultModel;

pub struct HttpResult {
    pub status_code: u16,
    pub description: String,
    pub result_type: Option<HttpResultModel>,
}

impl HttpResult {
    pub fn new(param_list: &ParamsList) -> Result<HttpResult, syn::Error> {
        let result = HttpResult {
            status_code: param_list
                .get_named_param("status_code")?
                .unwrap_as_number_value()?
                .as_u16(),
            description: param_list
                .get_named_param("description")?
                .unwrap_as_string_value()?
                .to_string(),
            result_type: HttpResultModel::new(param_list)?,
        };

        Ok(result)
    }

    /*
    pub fn new_n(src: Option<String>) -> Vec<HttpResult> {
        let mut result = vec![];

        if src.is_none() {
            return result;
        }

        for fields in JsonObjectsSimpleScanner::new(src.as_ref().unwrap().as_bytes()) {
            let mut status_code: Option<u16> = None;
            let mut description: Option<String> = None;
            let mut model: Option<String> = None;
            let mut model_as_array: Option<String> = None;

            for field in fields.split(',') {
                let kv: Vec<&str> = field.split(':').collect();

                if kv.len() < 2 {
                    continue;
                }

                match kv[0].trim() {
                    "status_code" => {
                        status_code = Some(kv[1].trim().parse::<u16>().unwrap());
                    }
                    "description" => {
                        description = Some(remove_quotes(kv[1].trim()));
                    }
                    "model" => {
                        model = Some(remove_quotes(kv[1].trim()));
                    }
                    "model_as_array" => {
                        model_as_array = Some(remove_quotes(kv[1].trim()));
                    }
                    _ => {
                        continue;
                    }
                }
            }

            if status_code.is_none() {
                panic!("status_code is not found");
            }

            if description.is_none() {
                panic!("description is not found");
            }

            result.push(HttpResult {
                status_code: status_code.unwrap(),
                description: description.unwrap(),
                result_type: ResultType::new(model, model_as_array),
            });
        }

        result
    }

     */
}
