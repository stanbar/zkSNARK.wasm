mod utils;
mod r1cs;

use std::fmt;
use wasm_bindgen::prelude::*;
use r1cs::*;

// // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// // allocator.
// #[cfg(feature = "wee_alloc")]
// #[global_allocator]
// static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    // Bindings must be named as their JS equivalent
    fn alert(s: &str);

    // A different name can be specified as long as the original name is passed to the macro.
    #[wasm_bindgen(js_name = prompt)]
    fn ask(s: &str) -> String;

    // Methods can be from any js namespace.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_unsafe(s: String);
}

fn log(s: String) {
    unsafe { log_unsafe(s) }
}

#[wasm_bindgen]
pub fn pass_two_arrays(input: Box<[JsValue]>, ops: Box<[JsValue]>) {
    let inputs: Vec<String> = input.iter().map(|e| e.as_string().unwrap()).collect();

    let ops: Vec<Operation> = ops
        .chunks(4)
        .map(|e| {
            let left = parse_operand(&e[2]).unwrap(); // think about handling this error
            let right = parse_operand(&e[3]).unwrap();

            Operation {
                operator: e[0].as_string().unwrap().into(),
                target: e[1].as_string().unwrap(),
                left,
                right,
            }
        })
        .inspect(|e| log(e.to_string()))
        .collect();

    let placements = get_var_placement(&inputs, &ops);
    // let (A, B, C) = flatcode_to_r1cs(inputs, ops);

    placements.iter().for_each(|e| log(e.to_string()));

    log(String::from("chunking ops"));
}

#[derive(Debug)]
pub enum JsValueType {
    String(String),
    Number(f64),
    Null,
    Undefined,
    Function,
}

impl JsValueType {
    pub fn from_jsvalue(jsvalue: &JsValue) -> Self {
        if jsvalue.is_string() {
            Self::String(jsvalue.as_string().unwrap())

        } else if jsvalue.as_f64().is_some() {
            Self::Number(jsvalue.as_f64().unwrap())

        } else if jsvalue.is_null() {
            Self::Null

        } else if jsvalue.is_undefined() {
            Self::Undefined

        } else if jsvalue.is_function() {
            Self::Function

        } else {
            panic!("Unknown type")
        }
    }
}

impl std::fmt::Display for JsValueType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::String(_) => write!(f, "string"),
            Self::Number(_) => write!(f, "number"),
            Self::Null => write!(f, "null"),
            Self::Undefined => write!(f, "undefined"),
            Self::Function => write!(f, "function"),
        }
    }
}


#[derive(Debug)]
pub enum Error {
    InvalidType(JsValueType),
    OperationTargetNotFound,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidType(type_name) => write!(f, "invalid type: `{}`", type_name),
            Self::OperationTargetNotFound => write!(f, "Operation target not found"),
        }
    }
}

fn parse_operand(js_operand: &JsValue) -> Result<Operand, Error> {
    let js_operand_type = JsValueType::from_jsvalue(js_operand);

    match js_operand_type {
        JsValueType::String(js_operand) => Ok(Operand::Identifier(js_operand)),
        JsValueType::Number(js_operand) => Ok(Operand::Number(js_operand)),
        type_name => Err(Error::InvalidType(type_name))
    }
}
