use serde_yaml::Value;

pub struct Templating {
    context: tera::Context,
}

impl Default for Templating {
    fn default() -> Self {
        Self {
            context: tera::Context::new(),
        }
    }
}

impl Templating {
    pub fn add_variable(&mut self, name: &str, value: Value) {
        self.context.insert(name.to_string(), &value);
    }

    pub fn process(&self, template: &str) -> Result<String, tera::Error> {
        tera::Tera::one_off(template, &self.context, false)
    }
}
