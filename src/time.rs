use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use rand::random_range;
use sdl2::{
    pixels::Color,
    rect::{Point, Rect},
    render::WindowCanvas,
};

pub struct Timer {
    start: Instant,
    end: Instant,
}

impl Timer {
    pub fn new(duration_ms: u32) -> Self {
        let now = Instant::now();
        Self {
            start: now,
            end: now + Duration::from_millis(duration_ms as u64),
        }
    }
    pub fn restart(&mut self) {
        let wait_time = self.wait_time();
        self.start = Instant::now();
        self.end = self.start + wait_time;
    }
    pub fn is_over(&self) -> bool {
        Instant::now() >= self.end
    }

    pub fn wait_time(&self) -> Duration {
        self.end - self.start
    }
    pub fn remaning_time(&self) -> Duration {
        self.end.saturating_duration_since(Instant::now())
    }
    pub fn draw_as_rect(
        &self,
        canvas: &mut WindowCanvas,
        rect: Rect,
        back_color: Color,
        fore_color: Color,
    ) {
        let remaining_time_percent =
            1. / self.wait_time().as_millis() as f32 * self.remaning_time().as_millis() as f32;
        let width = rect.width() as f32 * remaining_time_percent;
        let inner_rect = Rect::new(rect.x, rect.y, width as u32, rect.height());
        canvas.set_draw_color(back_color);
        canvas.fill_rect(rect).unwrap();
        canvas.set_draw_color(fore_color);
        canvas.fill_rect(inner_rect).unwrap();
    }

    pub fn draw_as_pixels(
        &self,
        canvas: &mut WindowCanvas,
        rect: Rect,
        max_points: i32,
        color: Color,
        start_at_percent: f32,
    ) {
        let mut points = Vec::new();
        let wait_ms = self.wait_time().as_millis();
        let remain_ms = self.remaning_time().as_millis();
        let start_at = (wait_ms as f32 * start_at_percent) as u128;
        if remain_ms < start_at {
            let percent = (1. / start_at as f32 * remain_ms as f32) * 100.;
            for _i in 0..max_points / (1 + percent as i32) {
                let x = random_range(rect.left()..rect.right()) as i32;
                let y = random_range(rect.top()..rect.bottom()) as i32;
                let p = Point::new(x, y);
                points.push(p);
            }
        }

        canvas.set_draw_color(color);
        canvas.draw_points(points.as_slice()).unwrap();
    }
}

pub struct FpsGuard {
    frame_start_time: Instant,
    code_time: Duration,
    frame_budget: Duration,
    wait_time: Duration,
    exceeded_time: Option<Instant>,
    delta_time: Duration,
}

impl FpsGuard {
    pub fn new(fps: u32) -> Self {
        let frame_start_time = Instant::now();
        let code_time = Duration::default();
        let frame_budget = Duration::from_millis(1000 / fps as u64);

        Self {
            frame_start_time,
            code_time,
            frame_budget,
            wait_time: frame_budget,
            exceeded_time: None,
            delta_time: Duration::default(),
        }
    }
    pub fn start_frame(&mut self) {
        self.frame_start_time = Instant::now()
    }
    pub fn end_frame(&mut self) {
        self.code_time = self.frame_start_time.elapsed();
        self.wait_time = self.frame_budget.saturating_sub(self.code_time);
        if self.code_time > self.frame_budget {
            self.exceeded_time = Some(Instant::now());
        }
        // loop {
        //     if Instant::now() > self.frame_start_time + self.frame_budget {
        //         break;
        //     }
        // }
        sleep(self.wait_time);
        // sleep(Duration::from_millis(1000 / 60));
        self.delta_time = self.frame_start_time.elapsed();
    }

    pub fn delta_ms(&self) -> u32 {
        self.delta_time.as_millis() as u32
    }

    pub fn dt(&self) -> f32 {
        self.delta_time.as_secs_f32()
    }

    pub fn draw(&self, cnv: &mut WindowCanvas, x: i32, y: i32) {
        const WIDTH: u32 = 20;
        const HEIGHT: u32 = 3;

        let occupancy_rate_in_percent =
            1. / self.frame_budget.as_millis() as f32 * self.code_time.as_millis() as f32;

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
    }
}
