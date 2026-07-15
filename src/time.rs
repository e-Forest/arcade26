use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use sdl3::{pixels::Color, rect::Rect, render::WindowCanvas};

pub struct Timer {
    current: Instant,
    duration: Duration,
}

impl Timer {
    pub fn new(duration_ms: u32) -> Self {
        Self {
            current: Instant::now(),
            duration: Duration::from_millis(duration_ms as u64),
        }
    }
    pub fn restart(&mut self) {
        self.current = Instant::now()
    }
    pub fn is_over(&self) -> bool {
        Instant::now().saturating_duration_since(self.current) > self.duration
    }
}

pub struct FpsGuard {
    frame_start_time: Instant,
    frame_duration: Duration,
    frame_duration_target: Duration,
    wait_time: Duration,
    exceeded_time: Option<Instant>,
}

impl FpsGuard {
    pub fn new(fps: u32) -> Self {
        let frame_start_time = Instant::now();
        let frame_duration = Duration::default();
        let frame_duration_target = Duration::from_millis(1000 / fps as u64);

        Self {
            frame_start_time,
            frame_duration,
            frame_duration_target,
            wait_time: frame_duration_target,
            exceeded_time: None,
        }
    }
    pub fn start_frame(&mut self) {
        self.frame_start_time = Instant::now()
    }
    pub fn end_frame(&mut self) {
        self.frame_duration = self.frame_start_time.elapsed();
        self.wait_time = self
            .frame_duration_target
            .saturating_sub(self.frame_duration);
        if self.frame_duration > self.frame_duration_target {
            self.exceeded_time = Some(Instant::now());
        }
        sleep(self.wait_time);
    }

    pub fn draw(&self, cnv: &mut WindowCanvas, x: i32, y: i32) {
        const WIDTH: u32 = 20;
        const HEIGHT: u32 = 3;

        let occupancy_rate_in_percent = 1. / self.frame_duration_target.as_millis() as f32
            * self.frame_duration.as_millis() as f32;

        cnv.set_draw_color(Color::WHITE);
        cnv.fill_rect(Rect::new(x, y, WIDTH, HEIGHT)).unwrap();
        cnv.set_draw_color(Color::BLACK);
        cnv.fill_rect(Rect::new(
            x,
            y,
            (occupancy_rate_in_percent * WIDTH as f32) as u32,
            HEIGHT,
        ))
        .unwrap();

        // - Wenn frame_time überschritten wird, wird die Anzeige für 1s rot -
        let is_exceeded = if let Some(exceeded_time) = self.exceeded_time {
            if exceeded_time.elapsed() < Duration::from_millis(1000) {
                true
            } else {
                false
            }
        } else {
            false
        };

        if is_exceeded {
            cnv.set_draw_color(Color::RED);
        } else {
            cnv.set_draw_color(Color::GREEN);
        }
        cnv.draw_rect(Rect::new(x, y, WIDTH, HEIGHT)).unwrap();
        // cnv.draw_rect(Rect::new(x + 1, y + 1, WIDTH - 2, HEIGHT - 2))
        //     .unwrap();
    }
}
