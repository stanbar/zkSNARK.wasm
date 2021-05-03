use super::utils::*;

pub fn r1cs_to_qap(
    a: &Vec<Vec<f64>>,
    b: &Vec<Vec<f64>>,
    c: &Vec<Vec<f64>>,
) -> (Vec<Vec<f64>>, Vec<Vec<f64>>, Vec<Vec<f64>>, Vec<f64>) {
    let (a, b, c) = (transpose(a), transpose(b), transpose(c));
    let (a, b, c) = (
        a.iter().map(lagrange_interop),
        b.iter().map(lagrange_interop),
        c.iter().map(lagrange_interop),
    );

    let z = (1..a.len())
        .fold(vec![1.0], |acc, i| mul_polys(acc, vec![-(i as f64), 1.0]));
    return (a.collect(), b.collect(), c.collect(), z);
}

fn lagrange_interop(vec: &Vec<f64>) -> Vec<f64> {
    use std::convert::TryInto;

    let o = vec.iter().enumerate().fold(vec![0.0], |acc, (i, x)| {
        add_polys(
            acc,
            mk_singleton(
                (i + 1).try_into().unwrap(),
                x,
                vec.len().try_into().unwrap(),
            ),
        )
    });

    o
}
/// Make a polynomial which is zero at {1, 2 ... total_points}, except for `point_loc` where the
/// value is `height`
fn mk_singleton(point_loc: i32, height: &f64, total_pts: i32) -> Vec<f64> {
    let fac = (1..total_pts)
        .filter(|i| *i != point_loc)
        .fold(1, |acc, i| acc * point_loc - i);

    let mut o = vec![height / (fac as f64)];

    for i in 1..total_pts {
        if i != point_loc {
            o = mul_polys(o, vec![-i as f64, 1.0])
        }
    }

    return o;
}

fn add_polys(a: Vec<f64>, b: Vec<f64>) -> Vec<f64> {
    let mut o = vec![0.0; std::cmp::max(a.len(), b.len())];
    a.iter().enumerate().for_each(|(i, v)| {
        o[i] += v;
    });
    b.iter().enumerate().for_each(|(i, v)| {
        o[i] += v;
    });
    o
}

fn sub_polys(a: Vec<f64>, b: Vec<f64>) -> Vec<f64> {
    let mut o = vec![0.0; std::cmp::max(a.len(), b.len())];
    a.iter().enumerate().for_each(|(i, v)| {
        o[i] += v;
    });
    b.iter().enumerate().for_each(|(i, v)| {
        o[i] += v * -1.0;
    });
    o
}

fn mul_polys(a: Vec<f64>, b: Vec<f64>) -> Vec<f64> {
    let mut o = vec![0.0; a.len() + b.len() - 1];
    for i in 0..a.len() {
        for j in 0..b.len() {
            o[i + j] += a[i] * b[j];
        }
    }
    o
}


pub fn create_solution_polynomials(r: Vec<f64>, a_p: Vec<Vec<f64>>, b_p: Vec<Vec<f64>>, c_p: Vec<Vec<f64>>) -> (Vec<f64>,Vec<f64>, Vec<f64>, Vec<f64>) {

    let a_poly = a_p.into_iter()
        .zip(r.into_iter())
        .fold(Vec::<f64>::with_capacity(r.len()), |acc, (a, rval)| add_polys(acc, mul_polys(vec![rval], a)) );

    let b_poly = b_p.into_iter()
        .zip(r.into_iter())
        .fold(Vec::<f64>::with_capacity(r.len()), |acc, (b, rval)| add_polys(acc, mul_polys(vec![rval], b)) );

    let c_poly = c_p.into_iter()
        .zip(r.into_iter())
        .fold(Vec::<f64>::with_capacity(r.len()), |acc, (c, rval)| add_polys(acc, mul_polys(vec![rval], c)) );


    let o = sub_polys(mul_polys(a_poly, b_poly), c_poly);
    //
    // add check

    (a_poly, b_poly, c_poly, o)
}
