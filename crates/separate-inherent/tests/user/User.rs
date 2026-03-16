impl User {
    fn new(name: String) -> Self {
        Self { name }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn set_name(&mut self, name: String) {
        self.name = name;
    }
}
