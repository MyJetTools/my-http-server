use super::HttpResult;

pub struct ApiData<'s> {
    pub controller: &'s str,
    pub description: &'s str,
    pub summary: &'s str,
    pub results: Option<Vec<HttpResult>>,
}

impl<'s> ApiData<'s> {
    pub fn new(
        controller: &'s str,
        attrs: &'s types_reader::ParamsList,
    ) -> Result<Self, syn::Error> {
        let description = attrs
            .get_named_param("description")?
            .unwrap_as_string_value()?
            .as_str();
        let summary = attrs
            .get_named_param("summary")?
            .unwrap_as_string_value()?
            .as_str();

        let results = if let Some(result) = attrs.try_get_named_param("result") {
            Some(result.unwrap_as_object_list()?)
        } else {
            None
        };

        let results = if let Some(http_results) = results {
            let mut result = Vec::new();

            for param_list in http_results {
                result.push(HttpResult::new(param_list)?);
            }

            Some(result)
        } else {
            None
        };

        Ok(Self {
            controller,
            description,
            summary,
            results,
        })
    }
}
