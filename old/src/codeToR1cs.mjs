import { parse } from 'acorn';

export default function codeToR1csWithInputs(code, inputVars) {
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
  printFlatcode(flatcode);

  const placements = getVarPlacement(inputs, flatcode);
  const { A, B, C } = flatcodeToR1cs(inputs, flatcode);
  const r = assignVariables(inputs, inputVars, flatcode);
  return {
    flatcode, placements, r, A, B, C,
  };
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

function printFlatcode(flatcode) {
  flatcode.forEach((fc, i) => {
    console.log(`${i}: ${fc.target} = ${fc.left} ${fc.op} ${fc.right}`);
  });
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

function Operation(op, target, left, right) {
  return {
    op, target, left, right,
  };
}

function flattenExpr(mksymbol, left, right) {
  if (right.type === 'Identifier') {
    // x = y
    return [Operation('set', left, right.name)];
  }
  if (right.type === 'Literal') {
    // x = 5
    return [Operation('set', left, right.raw)];
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
        return [Operation('set', left, 1)];
      } if (right.right.raw === 1) {
        return [Operation('set', left, flattenExpr(mksymbol, left, right.left))];
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
        out = [...out, Operation('*', nxt, latest, base)];
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
    return [...sub1, ...sub2, Operation(op, left, var1, var2)];
  }
  throw Error(`Bad expression ${right.type}`);
}

// Maps input, output and intermediate variables to indices
function getVarPlacement(inputs, flatcode) {
  const placements = ['~one', ...inputs, '~out'];
  flatcode.filter((code) => !inputs.includes(code.target) && code.target !== '~out')
    .forEach((code) => placements.push(code.target));

  return placements;
}

// Convert the flattened code generated above into a rank-1 constraint system
function flatcodeToR1cs(inputs, flatcode) {
  const varz = getVarPlacement(inputs, flatcode);
  const A = [];
  const B = [];
  const C = [];
  const used = new Map();
  inputs.forEach((input) => used.set(input, true));
  flatcode.forEach(({
    op, target, left, right,
  }) => {
    const a = new Array(varz.length).fill(0);
    const b = new Array(varz.length).fill(0);
    const c = new Array(varz.length).fill(0);

    used.set(target, true);
    if (target === 'set') {
      a[varz.indexOf(target)] += 1;
      insertVar(a, varz, left, used, true);
    } else if (op === '+' || op === '-') {
      c[varz.indexOf(target)] = 1;
      insertVar(a, varz, left, used, false);
      insertVar(a, varz, right, used, op === '-');
      b[0] = 1;
    } else if (op === '*') {
      c[varz.indexOf(target)] = 1;
      insertVar(a, varz, left, used, false);
      insertVar(b, varz, right, used, false);
    } else if (op === '/') {
      c[varz.indexOf(target)] = 1;
      insertVar(c, varz, left, used, false);
      a[varz.indexOf(target)] = 1;
      insertVar(b, varz, right, used, false);
    }
    A.push(a);
    B.push(b);
    C.push(c);
  });
  return { A, B, C };
}

// Adds a variable or number into one of the vectors; if it's a variable
// then the slot associated with that variable is set to 1, and if it's
// a number then the slot associated with 1 gets set to that number
function insertVar(arr, varz, variable, used, reverse) {
  if (typeof variable === 'string') {
    if (!used.has(variable)) {
      throw Error(`Using a variable before it is set! ${variable} ${JSON.stringify(used)}`);
    }
    /* eslint no-param-reassign: 0 */
    arr[varz.indexOf(variable)] += reverse ? -1 : 1;
  } else if (typeof variable === 'number') {
    /* eslint no-param-reassign: 0 */
    arr[0] += variable * (reverse ? -1 : 1);
  }
}

// Goes through flattened code and completes the input vector
function assignVariables(inputs, inputVars, flatcode) {
  const varz = getVarPlacement(inputs, flatcode);
  const assignment = new Array(varz.length).fill(0);
  assignment[0] = 1;
  inputVars.forEach((varr, index) => {
    assignment[index + 1] = varr;
  });

  flatcode.forEach(({
    op, target, left, right,
  }) => {
    if (op === 'set') {
      assignment[varz.indexOf(target)] = grabVar(varz, assignment, left);
    } else if (op === '+') {
      assignment[varz.indexOf(target)] = grabVar(varz, assignment, left)
        + grabVar(varz, assignment, right);
    } else if (op === '-') {
      assignment[varz.indexOf(target)] = grabVar(varz, assignment, left)
        - grabVar(varz, assignment, right);
    } else if (op === '*') {
      assignment[varz.indexOf(target)] = grabVar(varz, assignment, left)
        * grabVar(varz, assignment, right);
    } else if (op === '/') {
      assignment[varz.indexOf(target)] = grabVar(varz, assignment, left)
        / grabVar(varz, assignment, right);
    }
  });
  return assignment;
}

// Get a variable or number given an existing input vector
function grabVar(varz, assignment, varr) {
  if (typeof varr === 'string') {
    return assignment[varz.indexOf(varr)];
  } if (typeof varr === 'number') {
    return varr;
  }
  throw new Error(`What kind of expression is this? ${varr}`);
}
