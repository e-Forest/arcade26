use std::ops::{Add, Mul, Sub};

use rand::random_range;
use sdl3::rect::{Point, Rect};

const NORMALIZE_THRESHOLD: f32 = 0.1;

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn zero() -> Vec2 {
        Vec2::new(0., 0.)
    }

    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn as_point(&self) -> Point {
        Point::new(self.x as i32, self.y as i32)
    }

    pub fn from_point(point: Point) -> Self {
        Self {
            x: point.x as f32,
            y: point.y as f32,
        }
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn random_normalized() -> Self {
        let x = random_range(-1.0..1.0);
        let y = random_range(-1.0..1.0);
        let v = Vec2 { x, y };
        v.normalized()
    }

    pub fn normalized(&self) -> Self {
        let len = self.length();
        if len > NORMALIZE_THRESHOLD {
            Self {
                x: self.x / len,
                y: self.y / len,
            }
        } else {
            Self { x: 0.0, y: 0.0 }
        }
    }

    pub fn distance(&self, other: &Self) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    pub fn direction(&self, other: &Self) -> Self {
        (*other - *self).normalized()
    }

    pub fn lerp(self, other: Vec2, t: f32) -> Vec2 {
        Vec2 {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }

    pub fn delta(&self, other: Vec2) -> Vec2 {
        Vec2 {
            x: other.x - self.x,
            y: other.y - self.y,
        }
    }

    pub fn angle(&self) -> f32 {
        self.y.atan2(self.x).to_degrees()
    }
}

pub fn middle_direction(v1: Vec2, v2: Vec2) -> Vec2 {
    v1.add(v2).normalized()
}

impl Add for Vec2 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Vec2 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;
    fn mul(self, scalar: f32) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

pub struct AspectFittedRect {
    pub ratio_w: f32,
    pub ratio_h: f32,
    pub ratio: f32,
    pub rect: Rect,
}
pub fn get_aspect_fitted_rect(
    inner_w: u32,
    inner_h: u32,
    outer_w: u32,
    outer_h: u32,
) -> AspectFittedRect {
    let ratio_w = outer_w as f32 / inner_w as f32;
    let ratio_h = outer_h as f32 / inner_h as f32;
    let ratio = ratio_w.min(ratio_h);

    let box_w = (inner_w as f32 * ratio) as u32;
    let box_h = (inner_h as f32 * ratio) as u32;
    let box_left = ((outer_w - box_w) / 2) as i32;
    let box_top = ((outer_h - box_h) / 2) as i32;

    AspectFittedRect {
        ratio_w,
        ratio_h,
        ratio,
        rect: Rect::new(box_left, box_top, box_w, box_h),
    }
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

pub fn rect_shifted(rect: Rect, shift: Point) -> Rect {
    Rect::new(
        rect.x + shift.x,
        rect.y + shift.y,
        rect.width(),
        rect.height(),
    )
}

pub fn devide_rect(rect: Rect, rows: u32, cols: u32) -> Vec<Rect> {
    let mut out = Vec::new();
    let w = rect.width() / cols;
    let h = rect.height() / rows;
    for i in 0..rows * cols {
        let x = i % cols;
        let y = i / cols;
        out.push(Rect::new(
            rect.x + x as i32 * w as i32,
            rect.y + y as i32 * h as i32,
            w,
            h,
        ));
    }
    out
}
