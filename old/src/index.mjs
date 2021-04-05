import fs from 'fs';
import { Table } from 'console-table-printer';
import codeToR1csWithInputs from './codeToR1cs.mjs';
import {r1csToQap, createSolutionPolynomials, createDivisorPolynomial } from './qapCreator.mjs';

const sourceCode = fs.readFileSync('./input.js');

const {
  placements, flatcode, r, A, B, C,
} = codeToR1csWithInputs(sourceCode, [3]);

printWitness(r)
printVector("C", C)
printVector("A", A)
printVector("B", B)

const [Ap, Bp, Cp, Z] = r1csToQap(A,B,C)
const [Apoly, Bpoly, Cpoly, sol] = createSolutionPolynomials(r, Ap, Bp, Cp)
console.log("Apoly")
console.table(Apoly)
console.log("Bpoly")
console.table(Bpoly)
console.log("Cpoly")
console.table(Cpoly)
console.log("sol")
console.table(sol)
const [quot, rem] = createDivisorPolynomial(sol, Z)
console.log({quot, rem})

function printWitness(vector) {
  const rowS = {
    placement: '          witness',
    ...Object.fromEntries(vector.map((value, index) => [placements[index], value])),
  };

  const witnessTable = new Table({
    columns: [
      { name: 'placement' },
      ...placements.map((plm) => ({ name: plm })),
    ],
  });
  witnessTable.addRow(rowS);
  witnessTable.printTable();
}

function printVector(name, vector) {
  function operationToLine(index) {
    const op = flatcode[index];
    return `${op.target} = ${op.left} ${op.op} ${op.right}`;
  }
  const rowFlatcode = vector.map((a, index) => {
    const entries = r.map((v, index) => [placements[index], a[index]]);
    const obj = Object.fromEntries(entries);
    return { placement: operationToLine(index), ...obj };
  });
  console.log(name)
  const table = new Table({
    columns: [
      { name: 'placement' },
      ...placements.map((plm) => ({ name: plm })),
    ],
  });
  table.addRows(rowFlatcode);
  table.printTable();
}

