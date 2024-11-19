use std::collections::HashMap;

use my_json::json_reader::{JsonFirstLineIteratorFromSlice, JsonValueRef};

pub struct SignalRMessage<'s> {
    pub headers: Option<HashMap<String, String>>,
    pub invocation_id: Option<JsonValueRef<'s>>,
    pub target: JsonValueRef<'s>,
    pub arguments: JsonValueRef<'s>,
}

impl<'s> SignalRMessage<'s> {
    pub fn parse(json_first_line_reader: &'s JsonFirstLineIteratorFromSlice<'s>) -> Self {
        let mut invocation_id = None;
        let mut target = None;
        let mut arguments = None;

        while let Some(line) = json_first_line_reader.get_next() {
            let (name, value) = line.unwrap();

            match name.as_unescaped_str().unwrap() {
                "invocationId" => {
                    invocation_id = Some(value);
                }
                "arguments" => {
                    arguments = Some(value);
                }
                "target" => target = Some(value),
                _ => {}
            }
        }

        if target.is_none() {
            panic!("Target is not found");
        }

        if arguments.is_none() {
            panic!("Arguments is not found");
        }

        Self {
            headers: None,
            invocation_id: invocation_id.take(),
            target: target.take().unwrap(),
            arguments: arguments.take().unwrap(),
        }
    }

    pub fn get_target(&self) -> String {
        self.target.as_unescaped_str().unwrap().to_string()
    }

    pub fn get_arguments(&self) -> Vec<u8> {
        self.arguments.as_bytes().to_vec()
    }
}
