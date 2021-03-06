use super::Error;
use std::fmt;

#[derive(Clone)]
pub enum Operator {
    Plus,
    Minus,
    Multiply,
    Divide,
    Set,
}

impl From<String> for Operator {
    fn from(item: String) -> Self {
        use Operator::*;
        match item.as_str() {
            "+" => Plus,
            "-" => Minus,
            "*" => Multiply,
            "/" => Divide,
            "set" => Set,
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
            Set => "set",
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

pub fn flatcode_to_r1cs(
    inputs: Vec<String>,
    flatcode: Vec<Operation>,
) -> (Vec<Vec<f64>>, Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let varz = get_var_placement(&inputs, &flatcode);
    let mut a: Vec<Vec<f64>> = vec![];
    let mut b: Vec<Vec<f64>> = vec![];
    let mut c: Vec<Vec<f64>> = vec![];

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
            let mut a_temp = vec![0.0; varz.len()];
            let mut b_temp = vec![0.0; varz.len()];
            let mut c_temp = vec![0.0; varz.len()];
            used.insert(target.clone(), false);

            let target_index: usize = varz
                .iter()
                .position(|v| *v == target)
                .ok_or(Error::OperationTargetNotFound)
                .unwrap();

            match operator {
                Operator::Set => {
                    a_temp[target_index] += 1.0;
                    insert_var(&mut a_temp, &varz, left, &used, true);
                    b_temp[0] = 1.0;
                }
                Operator::Plus | Operator::Minus => {
                    c_temp[target_index] = 1.0;
                    insert_var(&mut a_temp, &varz, left, &used, false);
                    insert_var(
                        &mut a_temp,
                        &varz,
                        right,
                        &used,
                        matches!(operator, Operator::Minus),
                    );
                    b_temp[0] = 1.0;
                }
                Operator::Multiply => {
                    c_temp[target_index] = 1.0;
                    insert_var(&mut a_temp, &varz, left, &used, false);
                    insert_var(&mut b_temp, &varz, right, &used, false);
                }
                Operator::Divide => {
                    insert_var(&mut c_temp, &varz, left, &used, false);
                    a_temp[target_index] = 1.0;
                    insert_var(&mut b_temp, &varz, right, &used, false);
                }
            }
            a.push(a_temp);
            b.push(b_temp);
            c.push(c_temp);
        },
    );

    return (a, b, c);
}

/// Adds a variable or number into one of the vectors; if it's a variable
/// then the slot associated with that variable is set to 1, and if it's
/// a number then the slot associated with 1 gets set to that number
fn insert_var(
    arr: &mut Vec<f64>,
    varz: &Vec<String>,
    variable: Operand,
    used: &std::collections::HashMap<String, bool>,
    reverse: bool,
) {
    match variable {
        Operand::Identifier(identifier) => {
            if !used.contains_key(&identifier) {
                panic!("Using a variable before it is set!")
            }
            let var_index: usize = varz
                .iter()
                .position(|v| *v == identifier)
                .ok_or(Error::OperationTargetNotFound)
                .unwrap();
            arr[var_index] += if reverse { -1.0 } else { 1.0 }
        }
        Operand::Number(value) => arr[0] += value * if reverse { -1.0 } else { 1.0 },
    }
}

// Goes through flattened code and completes the input vector
pub fn assign_variables(inputs: &Vec<String>, input_vars: &Vec<f64>, flatcode: &Vec<Operation>) -> Vec<f64> {
    use std::iter;
    let varz = get_var_placement(inputs, flatcode);

    let mut assignment: Vec<f64> = iter::once(1.0)
        .chain(input_vars.clone().into_iter())
        .collect();


    flatcode.iter().for_each(|Operation {
             operator,
             target,
             left,
             right,
         }| {

            let target_index: usize = varz
                .iter()
                .position(|v| v == target)
                .ok_or(Error::OperationTargetNotFound)
                .unwrap();

             match operator {
                 Operator::Set => {
                     assignment[target_index] = grab_var(&varz, &assignment, left);
                 }
                 Operator::Plus => {
                     assignment[target_index] = grab_var(&varz, &assignment, left) + grab_var(&varz, &assignment, right) 
                 }
                 Operator::Minus => {
                     assignment[target_index] = grab_var(&varz, &assignment, left) - grab_var(&varz, &assignment, right) 
                 }
                 Operator::Multiply => {
                     assignment[target_index] = grab_var(&varz, &assignment, left) * grab_var(&varz, &assignment, right) 
                 }
                 Operator::Divide => {
                     assignment[target_index] = grab_var(&varz, &assignment, left) / grab_var(&varz, &assignment, right) 
                 }
             }

    });

    assignment
}


fn grab_var(varz: &Vec<String>, assignment: &Vec<f64>, varr: &Operand) -> f64 {
    match varr {
        Operand::Identifier(x) => {
            let var_index: usize = varz
                .iter()
                .position(|v| v == x)
                .ok_or(Error::OperationTargetNotFound)
                .unwrap();
            return assignment[var_index]
        }
        Operand::Number(x) => {
            return x.clone();
        }
    }
}
