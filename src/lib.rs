mod r1cs;
mod qap;
mod utils;

use r1cs::*;
use qap::*;
use std::fmt;
use wasm_bindgen::prelude::*;

// // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// // allocator.
// #[cfg(feature = "wee_alloc")]
// #[global_allocator]
// static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use js_sys::Array;

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
    // Methods can be from any js namespace.
    #[wasm_bindgen(js_namespace = console, js_name = table)]
    fn table_unsafe(s: Box<[f64]>);
    // Methods can be from any js namespace.
    #[wasm_bindgen(js_namespace = console, js_name = table)]
    fn table_2d_unsafe(s: Array);
}

fn log(s: String) {
    unsafe { log_unsafe(s) }
}

fn table(table: Vec<f64>) {
    unsafe { table_unsafe(table.into_boxed_slice()) }
}

fn table2d(table: Vec<Vec<f64>>) {
    unsafe {
        table_2d_unsafe(
            table
                .into_iter()
                .map(|e| e.into_iter().map(JsValue::from_f64).collect::<Array>())
                .collect(),
        )
    }
}

#[wasm_bindgen]
pub fn code_to_r1cs(input: Box<[JsValue]>, ops: Box<[JsValue]>, input_vars: Box<[JsValue]>) {
    let input_vars: Vec<f64> = input_vars.iter().map(|e| e.as_f64().unwrap()).collect();
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
    placements.iter().for_each(|e| log(e.to_string()));

    let (a, b, c) = flatcode_to_r1cs(inputs.clone(), ops.clone());
    let r = assign_variables(&inputs, &input_vars, &ops);

    table2d(a);
    table2d(b);
    table2d(c);

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
        type_name => Err(Error::InvalidType(type_name)),
    }
}
