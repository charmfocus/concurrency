use anyhow::anyhow;
use std::{
    fmt,
    ops::{Add, AddAssign, Mul},
    sync::mpsc,
    thread,
};

use crate::{dot_product, Vector};

const NUM_THREADS: usize = 4;

pub struct Matrix<T> {
    rows: usize,
    columns: usize,
    data: Vec<T>,
}
impl<T> Matrix<T> {
    pub fn new(data: impl Into<Vec<T>>, rows: usize, columns: usize) -> Self {
        Matrix {
            rows,
            columns,
            data: data.into(),
        }
    }
}

impl<T> fmt::Display for Matrix<T>
where
    T: fmt::Display,
{
    // display matrix a 2x3 as {1, 2 3, 4 5 6}, 3x2 as {1 2, 3 4, 5 6}
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{")?;
        for i in 0..self.rows {
            for j in 0..self.columns {
                write!(f, "{} ", self.data[i * self.columns + j])?;
                if j != self.columns - 1 {
                    write!(f, " ")?;
                }
            }
            if i != self.rows - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "}}")?;
        Ok(())
    }
}

impl<T> fmt::Debug for Matrix<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Matrix(rows={}, columns={}, {})",
            self.rows, self.columns, self
        )
    }
}

impl<T> Mul for Matrix<T>
where
    T: Copy + Add<Output = T> + AddAssign + Mul<Output = T> + Default + Send + 'static,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        self.multiply(&rhs).unwrap()
    }
}

impl<T> Matrix<T>
where
    T: Copy + Add<Output = T> + AddAssign + Mul<Output = T> + Default + Send + 'static,
{
    pub fn multiply(&self, other: &Matrix<T>) -> anyhow::Result<Matrix<T>> {
        if self.columns != other.rows {
            return Err(anyhow!("Matrix dimensions do not match"));
        }

        let senders = (0..NUM_THREADS)
            .map(|_| {
                let (tx, rx) = mpsc::channel::<Msg<T>>();
                thread::spawn(move || {
                    for msg in rx {
                        let val = dot_product(msg.input.row, msg.input.col)?;
                        if let Err(e) = msg.sender.send(MsgOutput::new(msg.input.idx, val)) {
                            eprintln!("Error sending message: {:?}", e);
                        }
                    }
                    Ok::<_, anyhow::Error>(())
                });
                tx
            })
            .collect::<Vec<_>>();
        // generate 4 threads witch receive msg and do dot product

        let matrix_len = self.rows * other.columns;
        let mut data = vec![T::default(); matrix_len];
        let mut receivers = Vec::with_capacity(matrix_len);

        for i in 0..self.rows {
            for j in 0..other.columns {
                let row = Vector::new(self.data[i * self.columns..(i + 1) * self.columns].to_vec());
                let col_data = other.data[j..]
                    .iter()
                    .step_by(other.columns)
                    .copied()
                    .collect::<Vec<_>>();
                let col = Vector::new(col_data);
                let idx = i * other.columns + j;
                let input = MsgInput::new(idx, row, col);
                let (tx, rx) = oneshot::channel();
                let msg = Msg::new(input, tx);
                if let Err(e) = senders[idx % NUM_THREADS].send(msg) {
                    eprintln!("Error sending message: {}", e);
                }
                receivers.push(rx);
            }
        }

        for rx in receivers {
            let output = rx.recv()?;
            data[output.idx] = output.val;
        }

        Ok(Matrix {
            rows: self.rows,
            columns: other.columns,
            data,
        })
    }
}

pub struct MsgInput<T> {
    pub idx: usize,
    pub row: Vector<T>,
    pub col: Vector<T>,
}

impl<T> MsgInput<T> {
    pub fn new(idx: usize, row: Vector<T>, col: Vector<T>) -> Self {
        Self { idx, row, col }
    }
}

pub struct MsgOutput<T> {
    pub idx: usize,
    pub val: T,
}

impl<T> MsgOutput<T> {
    pub fn new(idx: usize, val: T) -> Self {
        Self { idx, val }
    }
}

pub struct Msg<T> {
    input: MsgInput<T>,
    sender: oneshot::Sender<MsgOutput<T>>,
}

impl<T> Msg<T> {
    pub fn new(input: MsgInput<T>, sender: oneshot::Sender<MsgOutput<T>>) -> Self {
        Self { input, sender }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_multiply() -> anyhow::Result<()> {
        let a = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);
        let b = Matrix::new([1, 2, 3, 4, 5, 6], 3, 2);
        let c = a * b;
        assert_eq!(c.columns, 2);
        assert_eq!(c.rows, 2);
        assert_eq!(c.data, vec![22, 28, 49, 64]);
        assert_eq!(
            format!("{:?}", c),
            "Matrix(rows=2, columns=2, {22  28 , 49  64 })"
        );
        Ok(())
    }

    #[test]
    fn test_a_can_not_multiply_b() {
        let a = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);
        let b = Matrix::new([1, 2, 3, 4], 2, 2);
        let c = a.multiply(&b);
        assert!(c.is_err());
    }

    #[test]
    #[should_panic]
    fn test_a_can_not_multiply_b_panic() {
        let a = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);
        let b = Matrix::new([1, 2, 3, 4], 2, 2);
        let _c = a * b;
    }
}
