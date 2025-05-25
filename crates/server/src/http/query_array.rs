use std::{
    fmt::{Display, Write},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
pub struct QueryArray(String);

impl<I: Display> FromIterator<I> for QueryArray {
    fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
        let mut str = String::new();

        for val in iter {
            let _ = write!(str, "{val},");
        }

        Self(str)
    }
}

impl QueryArray {
    pub fn iter<I: FromStr>(&self) -> impl Iterator<Item = I> + Clone {
        self.0.split(',').filter_map(|x| x.parse().ok())
    }
}
