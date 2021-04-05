'use strict'
// A, B, C = matrices of m vectors of length n, where for each
// 0 <= i < m, we want to satisfy A[i] * B[i] - C[i] = 0
export function r1csToQap(A, B, C){
  A = transpose(A)
  B = transpose(B)
  C = transpose(C)
  const newA = A.map(lagrangeInterp)
  const newB = B.map(lagrangeInterp)
  const newC = C.map(lagrangeInterp)
  let Z = [1]
  for(let i = 1; i < A[0].length + 1; i++) {
    Z = multiplyPolys(Z, [-i, 1])
  }
  return [newA, newB, newC, Z]
}

function transpose(array) {
  return array[0].map((_, colIndex) => array.map(row => row[colIndex]));
}

// Assumes vec[0] = p(1), vec[1] = p(2), etc, tries to find p,
// expresses result as [deg 0 coeff, deg 1 coeff...]
function lagrangeInterp(vec){
  let o = []
  for (let i = 0; i < vec.length; i++){
    o = addPolys(o, mkSingleton(i + 1, vec[i], vec.length), false)
  }
  return o
}

// Make a polynomial which is zero at {1, 2 ... total_pts}, except
// for `point_loc` where the value is `height`
function mkSingleton(point_loc, height, total_pts){
  let fac = 1
  for (let i = 1; i < total_pts + 1; i++){
    if (i !== point_loc) {
      fac *= point_loc - i
    }
  }
  let o = [height * 1.0 / fac]
  console.log({fac,o})
  for (let i = 1; i < total_pts + 1; i++){
    if (i != point_loc) {
      o = multiplyPolys(o, [-i, 1])
      console.log({o})
    }
  }
  return o
}

// Multiply two polynomials
function multiplyPolys(a, b){
  const o =  new Array(a.length + b.length - 1).fill(0)
  for (let i = 0; i < a.length; i++){
    for (let j = 0; j < b.length; j++){
      o[i + j] += a[i] * b[j]
    }
  }
  return o
}


export function createSolutionPolynomials(r, newA, newB, newC){
  let Apoly = []
  for (const [rval, a] of zip(r, newA)){
    Apoly = addPolys(Apoly, multiplyPolys([rval], a), false)
  }
  let Bpoly = []
  for (const [rval, b] of zip(r, newB)){
    Bpoly = addPolys(Bpoly, multiplyPolys([rval], b), false)
  }
  let Cpoly = []
  for (const [rval, c] of zip(r, newC)){
    Cpoly = addPolys(Cpoly, multiplyPolys([rval], c), false)
  }
  let o = subtractPolys(multiplyPolys(Apoly, Bpoly), Cpoly)
  for (let i = 1; i < newA[0].length + 1; i++) {
    if (Math.abs(evalPoly(o, i)) >= 10**-10){
      throw Error("Invalid solution")
    } 
  }
  return [Apoly, Bpoly, Cpoly, o] 
}

function zip(a, b){
  return a.map((e, i) => [e, b[i]]);
}


export function createDivisorPolynomial(sol, Z){
  const [quot, rem] = divPolys(sol, Z)
  return [quot, rem]
}

// Divide a/b, return quotient and remainder
function divPolys(a, b){
  const o = new Array(a.length - b.length + 1).fill(0)
  let remainder = a
  let leading_fac, pos
  while (remainder.length >= b.length) {
    leading_fac = remainder[remainder.length - 1] / b[b.length - 1]
    pos = remainder.length - b.length
    o[pos] = leading_fac
    remainder = subtractPolys(remainder, multiplyPolys(b, [...new Array(pos).fill(0), leading_fac])).slice(0,-1)
  }
  return [o, remainder]
}

// Add two polynomials
function addPolys(a, b, subtract){
  const o = new Array(Math.max(a.length, b.length)).fill(0)
  for (let i = 0; i < a.length; i++){
    o[i] += a[i]
  }
  for (let i = 0; i < b.length; i++){
    o[i] += b[i] * (subtract ? -1 : 1) // Reuse the function structure for subtraction
  }

  return o
}

function subtractPolys(a, b){
    return addPolys(a, b, true)
}

// Evaluate a polynomial at a point
function evalPoly(poly, x){
  return poly.reduce((acc, v, i) =>  acc + v * x**i)
}
