use std::cell::RefCell;

pub struct Callback<Input, Output> {
    function: Option<RefCell<Box<dyn FnMut(Input) -> Output>>>,
}

impl<Input, Output> Callback<Input, Output> {
    pub fn new(function: Option<Box<dyn FnMut(Input) -> Output>>) -> Self {
        match function {
            Some(func) => Callback {
                function: Some(RefCell::new(func)),
            },
            None => Callback { function: None },
        }
    }
    pub fn fire(&self, args: Input) -> Option<Output> {
        match self.function.as_ref() {
            None => None,
            Some(func) => Some(func.borrow_mut()(args)),
        }
    }

    /// Allows the firing of a CB through an optional reference without
    /// consuming the underlying cb!
    pub fn fire_through_reference(cb_ref: &Option<Self>, args: Input) -> Option<Output> {
        match cb_ref {
            None => None,
            Some(cb_ref) => cb_ref.clone().fire(args),
        }
    }
}
