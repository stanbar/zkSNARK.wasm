# zkSNARK.js

JavaScript compiler for zkSNARK.
The compilation consist of two phases:
1. First it transforms js code to rank 1 constraint system (R1CS).
2. Then it transforms R1CS to Quadratic Arithmetic Program.

# Usage

Write JavaScript ES2020 source code in `input.js`, then execute compiler with
```
npm start
```

The compiler output to `output.zk`.

# Example

For following input JavaScript code:

`input.js`

```js
function qeval(x) { 
  y = x**5
  return y + x + 5
}
```

Compiler first flat it to simpler form:
```
0: sym_1 = x * x
1: sym_2 = sym_1 * x
2: sym_3 = sym_2 * x
3: y = sym_3 * x
4: sym_4 = y + x
5: ~out = sym_4 + 5
```

and output four vectors (rank-1 constaint system). 
Vector C corresponds to variables used on the left side of the operation.
Vector A corresponds to left operand, and vector B corresponds to right operand.
Vector S (witness) is the solution to the equation `S . C = S . A * S . B`, where `.` represents the dot product.

```
Witness:
┌───────────────────┬──────┬───┬──────┬───────┬───────┬───────┬─────┬───────┐
│         placement │ ~one │ x │ ~out │ sym_1 │ sym_2 │ sym_3 │   y │ sym_4 │
├───────────────────┼──────┼───┼──────┼───────┼───────┼───────┼─────┼───────┤
│           witness │    1 │ 3 │  251 │     9 │    27 │    81 │ 243 │   246 │
└───────────────────┴──────┴───┴──────┴───────┴───────┴───────┴─────┴───────┘
C:
┌───────────────────┬──────┬───┬──────┬───────┬───────┬───────┬───┬───────┐
│         placement │ ~one │ x │ ~out │ sym_1 │ sym_2 │ sym_3 │ y │ sym_4 │
├───────────────────┼──────┼───┼──────┼───────┼───────┼───────┼───┼───────┤
│     sym_1 = x * x │    0 │ 0 │    0 │     1 │     0 │     0 │ 0 │     0 │
│ sym_2 = sym_1 * x │    0 │ 0 │    0 │     0 │     1 │     0 │ 0 │     0 │
│ sym_3 = sym_2 * x │    0 │ 0 │    0 │     0 │     0 │     1 │ 0 │     0 │
│     y = sym_3 * x │    0 │ 0 │    0 │     0 │     0 │     0 │ 1 │     0 │
│     sym_4 = y + x │    0 │ 0 │    0 │     0 │     0 │     0 │ 0 │     1 │
│  ~out = sym_4 + 5 │    0 │ 0 │    1 │     0 │     0 │     0 │ 0 │     0 │
└───────────────────┴──────┴───┴──────┴───────┴───────┴───────┴───┴───────┘
A:
┌───────────────────┬──────┬───┬──────┬───────┬───────┬───────┬───┬───────┐
│         placement │ ~one │ x │ ~out │ sym_1 │ sym_2 │ sym_3 │ y │ sym_4 │
├───────────────────┼──────┼───┼──────┼───────┼───────┼───────┼───┼───────┤
│     sym_1 = x * x │    0 │ 1 │    0 │     0 │     0 │     0 │ 0 │     0 │
│ sym_2 = sym_1 * x │    0 │ 0 │    0 │     1 │     0 │     0 │ 0 │     0 │
│ sym_3 = sym_2 * x │    0 │ 0 │    0 │     0 │     1 │     0 │ 0 │     0 │
│     y = sym_3 * x │    0 │ 0 │    0 │     0 │     0 │     1 │ 0 │     0 │
│     sym_4 = y + x │    0 │ 1 │    0 │     0 │     0 │     0 │ 1 │     0 │
│  ~out = sym_4 + 5 │    5 │ 0 │    0 │     0 │     0 │     0 │ 0 │     1 │
└───────────────────┴──────┴───┴──────┴───────┴───────┴───────┴───┴───────┘
B:
┌───────────────────┬──────┬───┬──────┬───────┬───────┬───────┬───┬───────┐
│         placement │ ~one │ x │ ~out │ sym_1 │ sym_2 │ sym_3 │ y │ sym_4 │
├───────────────────┼──────┼───┼──────┼───────┼───────┼───────┼───┼───────┤
│     sym_1 = x * x │    0 │ 1 │    0 │     0 │     0 │     0 │ 0 │     0 │
│ sym_2 = sym_1 * x │    0 │ 1 │    0 │     0 │     0 │     0 │ 0 │     0 │
│ sym_3 = sym_2 * x │    0 │ 1 │    0 │     0 │     0 │     0 │ 0 │     0 │
│     y = sym_3 * x │    0 │ 1 │    0 │     0 │     0 │     0 │ 0 │     0 │
│     sym_4 = y + x │    1 │ 0 │    0 │     0 │     0 │     0 │ 0 │     0 │
│  ~out = sym_4 + 5 │    1 │ 0 │    0 │     0 │     0 │     0 │ 0 │     0 │
└───────────────────┴──────┴───┴──────┴───────┴───────┴───────┴───┴───────┘

```


Next step is to convert the R1CS to QAP form, which implements the same logic except using polynomials instead of dot product. 
