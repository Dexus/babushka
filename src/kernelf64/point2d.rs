use std::ops::{Add, Neg, Sub};
use num_traits::Zero;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

impl crate::point::Point2D for Point2D {
    type Value = f64;

    fn x(&self) -> Self::Value {
        self.x
    }

    fn y(&self) -> Self::Value {
        self.y
    }

    fn from_xy(x: Self::Value, y: Self::Value) -> Self {
        Self { x, y }
    }

    fn set_x(&mut self, x: Self::Value) {
        self.x = x;
    }

    fn set_y(&mut self, y: Self::Value) {
        self.y = y;
    }


}

impl Add for Point2D {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Point2D {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Neg for Point2D {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}


impl Zero for Point2D {
    fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    fn is_zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0
    }
}
