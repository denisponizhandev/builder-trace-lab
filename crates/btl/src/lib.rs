pub struct App {
    name: String
}

impl App {
    pub fn new() -> Self {
        Self {
            name: String::from("builder-trace-lab")
        }
    }

    pub fn start(&self) {
        println!("App {} has started!", &self.name);
    }
}
