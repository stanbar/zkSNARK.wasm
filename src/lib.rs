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

pub struct Operation {
    target: String,
    op: String,
    left: String,
    right: String
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
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: String);
}



#[wasm_bindgen]
pub fn pass_two_arrays(input: Box<[JsValue]>, ops: Box<[JsValue]>) {
    use std::iter;
    let inputs = input.iter().map(|e| e.as_string().unwrap());
    let placements: Vec<String> = 
         iter::once("~one".to_string())
        .chain(inputs.take(1))
        .chain(iter::once("~out".to_string()))
        .collect();


    placements.iter().for_each(|e| log(e.to_string()));

    log(String::from("chunking ops"));


    let ops: Vec<Operation> = ops.chunks(4)
        .map(|e| {
            let left = if e[2].is_string { e[2].as_string } else { e[2].as_f64 }
            Operation {
                op: String::from(e[0].as_string().unwrap()),
                target: String::from(e[1].as_string().unwrap()),
                left: String::from(e[2].as_string().unwrap()),
                right: String::from(e[3].as_string().unwrap()),
            }

        } 
        ).inspect(|e| log(e.to_string())).collect();


}


