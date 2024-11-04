use anyhow::Result;
use std::{
    fmt,
    ops::{Add, AddAssign, Mul},
};

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

impl<T> Matrix<T>
where
    T: fmt::Debug + Copy + Add<Output = T> + AddAssign + Mul<Output = T> + Default,
{
    pub fn multiply(&self, other: &Matrix<T>) -> Result<Matrix<T>> {
        if self.columns != other.rows {
            return Err(anyhow::anyhow!("Matrix dimensions do not match"));
        }

        let mut data = vec![T::default(); self.rows * other.columns];

        for i in 0..self.rows {
            for j in 0..other.columns {
                for k in 0..self.columns {
                    data[i * other.columns + j] +=
                        self.data[i * self.columns + k] * other.data[k * other.columns + j];
                }
            }
        }

        Ok(Matrix {
            rows: self.rows,
            columns: other.columns,
            data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_multiply() -> Result<()> {
        let a = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);
        let b = Matrix::new([1, 2, 3, 4, 5, 6], 3, 2);
        let c = a.multiply(&b)?;
        assert_eq!(c.columns, 2);
        assert_eq!(c.rows, 2);
        assert_eq!(c.data, vec![22, 28, 49, 64]);
        assert_eq!(
            format!("{:?}", c),
            "Matrix(rows=2, columns=2, {22  28 , 49  64 })"
        );
        Ok(())
    }
}
