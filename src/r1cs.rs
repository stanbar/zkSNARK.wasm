use std::fmt;
use super::Error;

#[derive(Clone)]
pub enum Operator {
    Plus,
    Minus,
    Multiply,
    Divide,
}

impl From<String> for Operator {
    fn from(item: String) -> Self {
        use Operator::*;
        match item.as_str() {
            "+" => Plus,
            "-" => Minus,
            "*" => Multiply,
            "/" => Divide,
            _ => panic!("Unknown operator"),
        }
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Operator::*;
        let sign = match self {
            Plus => "+",
            Minus => "-",
            Multiply => "*",
            Divide => "/",
        };

        write!(f, "{}", sign)
    }
}

#[derive(Clone)]
pub struct Operation {
    pub target: String,
    pub operator: Operator,
    pub left: Operand,
    pub right: Operand,
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} = {} {} {}",
            self.target, self.left, self.operator, self.right
        )
    }
}

#[derive(Clone)]
pub enum Operand {
    Number(f64),
    Identifier(String),
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Identifier(v) => write!(f, "{}", v),
            Self::Number(v) => write!(f, "{}", v),
        }
    }
}

/// Returns the order of variable identificators in following order:
/// [ ~one, ...inputs, ~out, ...vars ]
pub fn get_var_placement(inputs: &Vec<String>, flatcode: &Vec<Operation>) -> Vec<String> {
    use std::iter;

    iter::once("~one".to_string())
        .chain(inputs.clone())
        .chain(iter::once("~out".to_string()))
        .chain(
            flatcode
                .iter()
                .filter(|code| {
                    let target = &code.target;
                    !inputs.contains(&target) && target != "~out"
                })
                .map(|code| code.target.clone()),
        )
        .collect()
}

pub fn flatcode_to_r1cs(inputs: Vec<String>, flatcode: Vec<Operation>) -> Vec<String> {
    let varz = get_var_placement(&inputs, &flatcode);
    let a: Vec<i64> = vec![];
    let b: Vec<i64> = vec![];
    let c: Vec<i64> = vec![];

    let mut used = std::collections::HashMap::<String, bool>::new();
    inputs.into_iter().for_each(|v| {
        used.insert(v, true);
    });
    flatcode.into_iter().for_each(
        |Operation {
             operator,
             target,
             left,
             right,
         }| {
            let mut a_temp = vec![0; varz.len()];
            let mut b_temp = vec![0; varz.len()];
            let mut c_temp = vec![0; varz.len()];
            used.insert(target.clone(), false);

            let target_index: usize = varz.iter().position(|v| *v == target).ok_or(Error::OperationTargetNotFound).unwrap();

            if target == "set" {
                a_temp[target_index] += 1;
            }
            unimplemented!()
        },
    );

    unimplemented!()
}
