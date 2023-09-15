use rust_extensions::StrOrString;

pub struct YamlWriter {
    content: Vec<u8>,
    pub level: usize,
}

impl YamlWriter {
    pub fn new() -> Self {
        Self {
            content: Vec::new(),
            level: 0,
        }
    }

    fn fill_spaces(&mut self, offset: usize) {
        let offset = self.level + offset;
        if offset > 0 {
            for _ in 0..offset {
                self.content.push(32);
            }
        }
    }

    pub fn write(&mut self, field: &str, value: &str) {
        self.fill_spaces(0);

        self.content.extend_from_slice(field.as_bytes());
        self.content.extend_from_slice(": ".as_bytes());
        self.content.extend_from_slice(value.as_bytes());
        self.content.push(13);
        self.content.push(10);
    }

    pub fn write_empty(&mut self, field: &str) {
        self.fill_spaces(0);

        self.content.extend_from_slice(field.as_bytes());
        self.content.push(b':');

        self.content.push(13);
        self.content.push(10);
    }

    pub fn write_bool(&mut self, field: &str, value: bool) {
        self.fill_spaces(0);

        self.content.extend_from_slice(field.as_bytes());
        self.content.push(b':');
        self.content.push(32);

        if value {
            self.content.extend_from_slice("true".as_bytes());
        } else {
            self.content.extend_from_slice("false".as_bytes());
        }

        self.content.push(13);
        self.content.push(10);
    }

    pub fn write_array<'s, TIter>(&mut self, name: &str, values: TIter)
    where
        TIter: Iterator<Item = StrOrString<'s>>,
    {
        self.write_empty(name);

        for value in values {
            self.fill_spaces(1);

            self.content.extend_from_slice("- ".as_bytes());
            self.content.extend_from_slice(value.as_str().as_bytes());
            self.content.push(13);
            self.content.push(10);
        }
    }

    pub fn write_upper_level(&mut self, name: &str, level_up: impl Fn(&mut Self)) {
        let array = name.starts_with('-');
        self.write_empty(name);
        self.increase_level();
        if array {
            self.increase_level();
        }

        level_up(self);
        self.decrease_level();

        if array {
            self.decrease_level();
        }
    }

    pub fn write_upper_level_with_ctx<TCtx>(
        &mut self,
        name: &str,
        ctx: TCtx,
        level_up: impl Fn(TCtx, &mut Self) -> TCtx,
    ) -> TCtx {
        let array = name.starts_with('-');
        self.write_empty(name);
        self.increase_level();
        if array {
            self.increase_level();
        }

        let ctx = level_up(ctx, self);
        self.decrease_level();

        if array {
            self.decrease_level();
        }
        ctx
    }

    pub fn write_upper_level_with_value(
        &mut self,
        name: &str,
        value: StrOrString,
        level_up: impl Fn(&mut Self),
    ) {
        let array = name.starts_with('-');
        self.write(name, value.as_str());
        self.increase_level();
        if array {
            self.increase_level();
        }

        level_up(self);
        self.decrease_level();

        if array {
            self.decrease_level();
        }
    }

    fn increase_level(&mut self) {
        self.level += 1;
    }

    fn decrease_level(&mut self) {
        if self.level == 0 {
            println!("Content: {}", String::from_utf8_lossy(&self.content));
            panic!("Somehow level is 0");
        }
        self.level -= 1;
    }

    pub fn build(self) -> Vec<u8> {
        self.content
    }
}
