import * as wasm from "wasm-game-of-life";
import {parse} from 'acorn';

// wasm.greet("zk-snark.wasm");

const input = document.getElementById("inCode")

const btnCompile = document.getElementById("btnCompile")
btnCompile.addEventListener("click", () => {
  const code = input.value
  const {inputs, flatcode} = parseCode(code)

  const operations = flatcode.map(fc => fc.toArray()).flat(1)

  wasm.pass_two_arrays(inputs, operations)

})

class Operation{
  constructor(op, target, left, right) {
    this.op = op;
    this.target = target;
    this.left = left;
    this.right = right;
  }

  toString() {
    return `${this.target} = ${this.left} ${this.op} ${this.right}`
  }
  toArray() {
    return [this.op, this.target, this.left, this.right]
  }
}

function parseCode(code) {

  let nextSymbol = 0;

  // Generate a dummy variable
  const mksymbol = () => {
    nextSymbol += 1;
    return `sym_${nextSymbol}`;
  };
  const parsed = parse(code, { ecmaVersion: 2020 });

  const { inputs, body } = extractInputsAndBody(parsed.body);
  console.log({inputs})

  const flatcode = flattenBody(mksymbol, body);
  return {inputs, flatcode }
}

function extractInputsAndBody(code) {
  const inputs = [];

  code.forEach((node) => {
    node.params.forEach((param) => { inputs.push(param.name); });
  });
  let returned = false;

  const body = [];
  const block = code[0].body;

  if (block.type !== 'BlockStatement') {
    throw new Error('Only block statement supported');
  }

  block.body.forEach((statement) => {
    if (statement.type !== 'ExpressionStatement' && statement.type !== 'ReturnStatement') {
      throw new Error('Unsupported statement');
    }
    if (returned) {
      throw new Error('Return has to be the last expression');
    }
    if (statement.type === 'ExpressionStatement') {
      body.push(statement);
    }
    if (statement.type === 'ReturnStatement') {
      body.push(statement);
      returned = true;
    }
  });

  return { inputs, body };
}

function flattenBody(mksymbol, body) {
  let output = [];
  body.forEach((statement) => {
    output = [...output, flattenStatement(mksymbol, statement)];
  });
  return output.flat(1);
}

function flattenStatement(mksymbol, statement) {
  if (statement.type === 'ReturnStatement') {
    return flattenExpr(mksymbol, '~out', statement.argument);
  }
  if (statement.type === 'ExpressionStatement') {
    return flattenExpr(mksymbol, statement.expression.left.name, statement.expression.right);
  }

  throw new Error('Unsupported statement type');
}


function flattenExpr(mksymbol, left, right) {
  if (right.type === 'Identifier') {
    // x = y
    return [new Operation('set', left, right.name)];
  }
  if (right.type === 'Literal') {
    // x = 5
    return [new Operation('set', left, right.raw)];
  }
  if (right.type === 'BinaryExpression') {
    // x = y (op) z
    // Or, for that matter, x = y (op) 5
    let op;
    if (['+', '-', '*', '/'].includes(right.operator)) {
      op = right.operator;
    } else if (right.operator === '**') {
      if (right.right.type !== 'Literal') {
        throw Error('Can not rise to power of variable');
      }
      if (right.right.raw === 0) {
        return [new Operation('set', left, 1)];
      } if (right.right.raw === 1) {
        return [new Operation('set', left, flattenExpr(mksymbol, left, right.left))];
      }
      // x**n
      let nxt;
      let base;
      let out;
      if (right.left.type === 'Identifier' || right.left.type === 'Literal') {
        nxt = right.left.type === 'Identifier' ? right.left.name : Number(right.left.raw);
        base = nxt;
        out = [];
      } else {
        nxt = mksymbol();
        base = nxt;
        out = flattenExpr(mksymbol, base, right.left);
      }

      let latest;
      for (let i = 1; i < right.right.raw; i += 1) {
        latest = nxt;
        nxt = i === right.right.raw - 1 ? left : mksymbol();
        out = [...out, new Operation('*', nxt, latest, base)];
      }
      return out;

      // Exponentiation gets compiled to repeat multiplication,
      // requires constant exponent
    } else {
      throw Error(`Bad operation: ${right.type}`);
    }

    let var1;
    let sub1;
    // If the subexpression is a variable or a number, then include it directly
    if (right.left.type === 'Identifier' || right.left.type === 'Literal') {
      var1 = right.left.type === 'Identifier' ? right.left.name : Number(right.left.raw);
      sub1 = [];
    } else {
      // If one of the subexpressions is itself a compound expression, recursively
      // apply this method to it using an intermediate variable
      var1 = mksymbol();
      sub1 = flattenExpr(mksymbol, var1, right.left);
    }

    let var2;
    let sub2;
    if (right.right.type === 'Identifier' || right.right.type === 'Literal') {
      var2 = right.right.type === 'Identifier'
        ? right.right.name : Number(right.right.raw);
      sub2 = [];
    } else {
      var2 = mksymbol();
      sub2 = flattenExpr(mksymbol, var2, right.right);
    }
    // Last expression represents the assignment; sub1 and sub2 represent the
    // processing for the subexpression if any
    return [...sub1, ...sub2, new Operation(op, left, var1, var2)];
  }
  throw Error(`Bad expression ${right.type}`);
}
