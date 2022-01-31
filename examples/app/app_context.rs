use rust_extensions::ApplicationStates;

pub struct AppContext;

impl AppContext {
    pub fn new() -> Self{
        Self{}
    }
}

impl ApplicationStates for AppContext {
    fn is_initialized(&self) -> bool {
        true
    }

    fn is_shutting_down(&self) -> bool {
        false
    }
}
