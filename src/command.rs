pub enum ParamValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String)
}

pub struct Parameter {
    name: String,
    description: String,
    param_type: String,
    required: bool,
    default_value: Option<ParamValue>
}

pub struct Command {
    name: String,
    description: String,
    tags: Vec<String>,
    package: String,
    parameters: Vec<Parameter>,
}
