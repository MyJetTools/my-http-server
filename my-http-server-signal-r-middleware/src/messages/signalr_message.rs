use std::collections::HashMap;

use my_json::json_reader::JsonFirstLineReader;
use rust_extensions::array_of_bytes_iterator::SliceIterator;

pub struct SignalRMessage<'s> {
    pub headers: Option<HashMap<String, String>>,
    pub invocation_id: Option<&'s str>,
    pub target: &'s str,
    pub arguments: &'s [u8],
}

impl<'s> SignalRMessage<'s> {
    pub fn parse(json_first_line_reader: &'s mut JsonFirstLineReader<SliceIterator>) -> Self {
        let mut invocation_id = None;
        let mut target = None;
        let mut arguments = None;

        while let Some(line) = json_first_line_reader.get_next() {
            let line = line.unwrap();

            match line.name.as_unescaped_name(json_first_line_reader).unwrap() {
                "invocationId" => {
                    invocation_id = Some(line.value);
                }
                "arguments" => {
                    arguments = Some(line.value);
                }
                "target" => target = Some(line.value),
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
            invocation_id: if let Some(invocation_id) = invocation_id.take() {
                Some(
                    invocation_id
                        .as_unescaped_str(json_first_line_reader)
                        .unwrap(),
                )
            } else {
                None
            },
            target: target
                .take()
                .unwrap()
                .as_unescaped_str(json_first_line_reader)
                .unwrap()
                .into(),
            arguments: arguments.take().unwrap().as_bytes(json_first_line_reader),
        }
    }
}
