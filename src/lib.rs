mod utils;

use wasm_bindgen::prelude::*;
use std::fmt;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;



#[wasm_bindgen]
pub fn compile(code: &str) {
    alert(&format!("Compiling, {}!", code));
}

enum Operand {
    Number(f64),
    Identifier(String)
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Operand::Identifier(x) => write!(f, "{}", x),
            Operand::Number(x) => write!(f, "{}", x),
        }
    }
}

pub struct Operation {
    target: String,
    op: String,
    left: Operand,
    right: Operand
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} = {} {} {}", self.target, self.left, self.op, self.right)
    }
}

#[wasm_bindgen]
extern {
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
    unsafe {
        log_unsafe(s)
    }
}


#[wasm_bindgen]
pub fn pass_two_arrays(input: Box<[JsValue]>, ops: Box<[JsValue]>) {
        let inputs: Vec<String> = input.iter().map(|e| e.as_string().unwrap()).collect();


        let ops: Vec<Operation> = ops.chunks(4)
            .map(|e| {
                let left = parse_operand(&e[2]);
                let right = parse_operand(&e[3]);

                Operation {
                    op: String::from(e[0].as_string().unwrap()),
                    target: String::from(e[1].as_string().unwrap()),
                    left,
                    right,
                }
            })
        .inspect(|e| log(e.to_string()))
            .collect();

        let placements = get_var_placement(&inputs, ops);

        placements.iter().for_each(|e| log(e.to_string()));

        log(String::from("chunking ops"));

}

/*
 * Returns the order of variable identificators in following order:
 * [ ~one, ...inputs, ~out, ...vars ]
 */
fn get_var_placement(inputs: &Vec<String>, flatcode: Vec<Operation>) -> Vec<String>{
    use std::iter;
    let mut placements: Vec<String> = 
        iter::once("~one".to_string())
        .chain(inputs)
        .chain(iter::once("~out".to_string()))
        .collect();

    flatcode.iter()
        .filter(|code| !inputs.contains(&code.target) && code.target != "~out")
        .for_each(|code| placements.push(code.target));

    return placements
}

fn parse_operand(js_operand: &JsValue) -> Operand {
    if js_operand.is_null() || js_operand.is_undefined() {
        panic!("Can not parse null or undefined operand")
    } else if js_operand.is_function() {
        panic!("Can not parse function as an operand")
    } else if js_operand.is_string() {
        Operand::Identifier(js_operand.as_string().expect("Could not parse operand as string"))
    } else {
        Operand::Number(js_operand.as_f64().expect("Could not parse operand as f64"))
    }
}


