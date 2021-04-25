use std::cmp;

pub fn r1cs_to_qap(A: Vec<Vec<f64>>,B: Vec<Vec<f64>>,C: Vec<Vec<f64>>) {
    let A = transpose(A);
    let B = transpose(B);
    let C = transpose(C);
    let newA = A.iter().map();

}

fn transpose<T>(v: Vec<Vec<T>>) -> Vec<Vec<T>>
where
    T: Clone,
{
    assert!(!v.is_empty());
    (0..v[0].len())
        .map(|i| v.iter().map(|inner| inner[i].clone()).collect::<Vec<T>>())
        .collect()
}

fn lagrange_interop(vec: Vec<f64>) -> Vec<f64> {

    vec.iter().map(|e| add_polys())

}

fn add_polys(a: Vec<f64>, b: Vec<f64>, substract: bool) -> Vec<f64> {
    let mut o = vec![0.0; cmp::min(a.len(),b.len())];
    a.iter().enumerate().for_each(|( i, v )| {
        o[i] += v;
    });
    b.iter().enumerate().for_each(|( i, v )| {
        o[i] += v * if substract { -1.0 } else { 1.0 };
    });
    o
}
