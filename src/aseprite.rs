use std::{
    fs,
    time::{Duration, Instant},
};

use sdl2::{
    rect::{Point, Rect},
    render::{Texture, WindowCanvas},
};
use serde::Deserialize;

use crate::math::Vec2;

#[derive(Debug, Clone)]
pub struct AsePlayer {
    data: AsepriteData,
    current_tag_index: usize,
    current_frame_index: usize,
    current_frame_index_old: usize,
    timer: Instant,
    is_finished: bool,
}

impl AsePlayer {
    pub fn from_json(asset_path: &str) -> Self {
        let mut data = AsepriteData::from_json(asset_path);
        if data.meta.tags.is_empty() {
            let (from, to) = (0, data.frames.len() - 1);
            data.meta.tags.push(Tag {
                name: "default".to_string(),
                from,
                to,
            });
        }
        let current_tag_index = 0;
        Self {
            data,
            current_tag_index,
            current_frame_index: 0,
            current_frame_index_old: 0,
            timer: Instant::now(),
            is_finished: false,
        }
    }
    pub fn play_tag(&mut self, tag_name: &str, looping: bool) {
        let tag_index = self
            .data
            .meta
            .tags
            .iter()
            .position(|x| x.name == tag_name)
            .expect("! Tag-Name existiert nicht");
        if self.current_tag_index != tag_index {
            self.timer = Instant::now();
            self.current_frame_index = self.current_tag().from;
            self.current_tag_index = tag_index;
        }
        self.play(looping);
    }

    fn play(&mut self, looping: bool) {
        self.is_finished = false;
        self.current_frame_index_old = self.current_frame_index;

        let (i_min, i_max) = (self.current_tag().from, self.current_tag().to);

        if !(i_min..=i_max).contains(&self.current_frame_index) {
            self.current_frame_index = i_min;
        }

        let duration = Duration::from_millis(self.current_frame().duration as u64);

        if Instant::now() >= self.timer + duration {
            self.timer = Instant::now();
            self.current_frame_index += 1;
            if self.current_frame_index > i_max {
                self.is_finished = true;
                if looping {
                    self.current_frame_index = i_min;
                } else {
                    self.current_frame_index = i_max;
                }
            }
        }
    }

    pub fn just_frame_index(&self, index: usize) -> bool {
        self.current_frame_index == index && self.current_frame_index_old != index
    }

    pub fn current_frame_rect(&self) -> Rect {
        self.current_frame().frame.as_sdl_rect()
    }
    pub fn frame_rect_by_index(&self, index: usize) -> Rect {
        self.frame_by_index(index).frame.as_sdl_rect()
    }
    fn current_frame(&self) -> &Frame {
        self.frame_by_index(self.current_frame_index)
    }
    fn frame_by_index(&self, index: usize) -> &Frame {
        self.data
            .frames
            .get(index)
            .expect("! FrameIndex existiert nicht")
    }
    fn current_tag(&self) -> &Tag {
        self.data
            .meta
            .tags
            .get(self.current_tag_index)
            .expect("! TagIndex existiert nicht")
    }
    pub fn current_frame_index(&self) -> usize {
        self.current_frame_index
    }
    pub fn draw_current_frame(
        &self,
        cnv: &mut WindowCanvas,
        pos: Vec2,
        texture: &Texture,
        anchor: AnchorPosition,
        fliped: bool,
    ) {
        let src = self.current_frame_rect();
        draw_texture(cnv, pos, texture, Some(src), anchor, fliped);
    }
    pub fn draw_frame_by_index(
        &self,
        cnv: &mut WindowCanvas,
        pos: Vec2,
        texture: &Texture,
        anchor: AnchorPosition,
        index: usize,
        fliped: bool,
    ) {
        let src = self.frame_rect_by_index(index);
        draw_texture(cnv, pos, texture, Some(src), anchor, fliped);
    }
    pub fn is_finished(&self) -> bool {
        self.is_finished
    }
}

#[derive(Debug, Clone, Deserialize)]
struct AsepriteData {
    frames: Vec<Frame>,
    meta: Meta,
}
impl AsepriteData {
    /// Aseprite-Spritesheet muss als Array exportiert sein
    fn from_json(path: &str) -> Self {
        println!("öffne '{}'...", path);
        let json_string = fs::read_to_string(path).expect("! Konnte json nicht lesen");
        serde_json::from_str(&json_string).expect("! Konnte AsepriteData nicht erstellen - Prüfe ob Spritesheet als 'Arra' exportiert wurde")
    }
}

#[derive(Debug, Clone, Deserialize)]
struct Frame {
    frame: SerdeRect,
    duration: u32,
}

#[derive(Debug, Deserialize, Clone, Copy)]
struct SerdeRect {
    x: i32,
    y: i32,
    w: u32,
    h: u32,
}
impl SerdeRect {
    fn as_sdl_rect(&self) -> Rect {
        Rect::new(self.x, self.y, self.w, self.h)
    }
}

#[derive(Debug, Clone, Deserialize)]
struct Meta {
    #[serde(rename = "frameTags")]
    tags: Vec<Tag>,
}

#[derive(Debug, Clone, Deserialize)]
struct Tag {
    name: String,
    from: usize,
    to: usize,
}

#[derive(Clone, Copy)]
pub enum AnchorPosition {
    Center,
    TopLeft,
    BottomCenter,
    Point(Point),
}

pub fn draw_texture(
    cnv: &mut WindowCanvas,
    pos: Vec2,
    texture: &Texture,
    src: Option<Rect>,
    anchor: AnchorPosition,
    fliped: bool,
) {
    let src = if let Some(src) = src {
        src
    } else {
        let q = texture.query();
        Rect::new(0, 0, q.width, q.height)
    };
    let (sw, sh) = (src.width(), src.height());
    let (shift_top, shift_left) = match anchor {
        AnchorPosition::TopLeft => (0, 0),
        AnchorPosition::Center => (sw as i32 / 2, sh as i32 / 2),
        AnchorPosition::BottomCenter => (sw as i32 / 2, sh as i32),
        AnchorPosition::Point(point) => (point.x, point.y),
    };
    // let center = FPoint::new(shift_left as f32, shift_top as f32);
    cnv.copy_ex(
        texture,
        src,
        Rect::new(
            // pos.x as i32,
            // pos.y as i32,
            pos.x as i32 - shift_top,
            pos.y as i32 - shift_left,
            src.width(),
            src.height(),
        ),
        0.,
        None,
        fliped,
        false,
    )
    .unwrap();
}
