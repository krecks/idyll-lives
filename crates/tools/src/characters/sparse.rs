//! Compressed sparse rows (scipy's `csr_matrix` as subdiv.py and body.py use it): built from
//! triplets with duplicates summed, products, and application to per-vertex attributes.

use nalgebra::DMatrix;

use super::V3;

#[derive(Clone, Debug)]
pub struct Csr {
    pub rows: usize,
    pub cols: usize,
    /// Per row: (column, value), columns increasing.
    pub data: Vec<Vec<(usize, f64)>>,
}

impl Csr {
    /// `coo_matrix((vals, (rows, cols)), shape).tocsr()`: duplicates summed in the order given.
    pub fn from_triplets(rows: usize, cols: usize, trips: &[(usize, usize, f64)]) -> Self {
        let mut data: Vec<Vec<(usize, f64)>> = vec![Vec::new(); rows];
        for &(r, c, v) in trips {
            data[r].push((c, v));
        }
        for row in &mut data {
            row.sort_by_key(|e| e.0);
            let mut out: Vec<(usize, f64)> = Vec::with_capacity(row.len());
            for &(c, v) in row.iter() {
                match out.last_mut() {
                    Some(last) if last.0 == c => last.1 += v,
                    _ => out.push((c, v)),
                }
            }
            *row = out;
        }
        Csr { rows, cols, data }
    }

    /// `self @ other`.
    pub fn matmul(&self, other: &Csr) -> Csr {
        assert_eq!(self.cols, other.rows);
        let mut acc = vec![0.0; other.cols];
        let mut used = vec![false; other.cols];
        let mut data = Vec::with_capacity(self.rows);
        for row in &self.data {
            let mut cols = Vec::new();
            for &(k, a) in row {
                for &(c, b) in &other.data[k] {
                    if !used[c] {
                        used[c] = true;
                        cols.push(c);
                    }
                    acc[c] += a * b;
                }
            }
            cols.sort_unstable();
            let out: Vec<(usize, f64)> = cols.iter().map(|&c| (c, acc[c])).collect();
            for &c in &cols {
                acc[c] = 0.0;
                used[c] = false;
            }
            data.push(out);
        }
        Csr {
            rows: self.rows,
            cols: other.cols,
            data,
        }
    }

    /// The first `n` rows (`M[:n]`).
    pub fn head(&self, n: usize) -> Csr {
        Csr {
            rows: n,
            cols: self.cols,
            data: self.data[..n].to_vec(),
        }
    }

    /// `M @ x` of points.
    pub fn apply(&self, x: &[V3]) -> Vec<V3> {
        self.data
            .iter()
            .map(|row| {
                let mut s = V3::zeros();
                for &(c, a) in row {
                    s += x[c] * a;
                }
                s
            })
            .collect()
    }

    /// `M @ x` of a scalar per vertex.
    pub fn apply1(&self, x: &[f64]) -> Vec<f64> {
        self.data
            .iter()
            .map(|row| row.iter().map(|&(c, a)| a * x[c]).fold(0.0, |s, v| s + v))
            .collect()
    }

    /// `M @ x` of K values per vertex.
    pub fn apply_n<const K: usize>(&self, x: &[[f64; K]]) -> Vec<[f64; K]> {
        self.data
            .iter()
            .map(|row| {
                let mut s = [0.0; K];
                for &(c, a) in row {
                    for k in 0..K {
                        s[k] += a * x[c][k];
                    }
                }
                s
            })
            .collect()
    }

    pub fn to_dense(&self) -> DMatrix<f64> {
        let mut m = DMatrix::zeros(self.rows, self.cols);
        for (r, row) in self.data.iter().enumerate() {
            for &(c, v) in row {
                m[(r, c)] = v;
            }
        }
        m
    }
}
