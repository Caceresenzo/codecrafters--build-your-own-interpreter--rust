use std::{cell::RefCell, rc::Rc};

#[derive(Debug, PartialEq)]
pub struct Class {
    name: String,
}

impl Class {
    pub fn new(name: String) -> Self {
        Class { name }
    }

    pub fn as_str(&self) -> String {
        return self.name.clone();
    }
}

#[derive(Debug, PartialEq)]
pub struct Instance {
    class: Rc<RefCell<Class>>,
}

impl Instance {
    pub fn new(class: Rc<RefCell<Class>>) -> Self {
        Instance { class }
    }

    pub fn as_str(&self) -> String {
        return format!("{} instance", self.class.borrow().name);
    }
}
