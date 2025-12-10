use crate::dag::{Value, Variable};

#[derive(Debug)]
pub struct Intervention {
    pub variable: Variable,
    pub value: Value,
}

impl Intervention {
    pub fn new(variable: Variable, value: Value) -> Self {
        Intervention { variable, value }
    }
}

#[macro_export]
macro_rules! intervene {
    ( $( $var:tt : $val:expr ),* $(,)? ) => {
        vec![
            $(
                Intervention::new($var.into(), $val.into()),
            )*
        ]
    };
}