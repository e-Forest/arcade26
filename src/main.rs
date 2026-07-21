use rand::random_range;
use sdl3::Sdl;
use sdl3::image::LoadTexture;
use sdl3::pixels::Color;
use sdl3::rect::{Point, Rect};
use sdl3::render::{BlendMode, ScaleMode, Texture, TextureCreator};
use sdl3::video::{Display, DisplayMode, FullscreenType, WindowContext, WindowPos};
use sdl3::{keyboard::Keycode, render::WindowCanvas};
use std::thread::sleep;
use std::time::Instant;
use std::{collections::HashMap, time::Duration};

use gilrs::Button;

pub mod arcadeinput;
use arcadeinput::*;

pub mod math;
use math::*;

pub mod aseprite;
use aseprite::*;

use crate::overworld::OverWorld;

pub mod overworld;
use overworld::*;

pub mod ballgame;
use ballgame::*;

pub mod fightgame;
use fightgame::*;

pub mod jumpgame;
use jumpgame::*;

pub mod time;
use time::*;

pub mod player;
use player::*;

pub const DEBUGMODE: bool = false;

pub const VIRTUAL_WIDHT: u32 = 320; // 1920/6
pub const VIRTUAL_HEIGHT: u32 = 180; // 1080/6

pub const ARROW_LIFETIME: Duration = Duration::from_millis(600);
pub const ARROW_SPEED: f32 = 3.;
pub const ARROW_SPAWN_DISTANCE: f32 = 10.;

pub const PARTICLE_LIFETIME_MAX_MS: u32 = 600;

pub const STUNNING_TIME: u64 = 300;
pub const STUNNING_SPEED_ARROW_HIT: f32 = 1.;
pub const STUNNING_SPEED_DASH_HIT: f32 = 1.6;

pub const DASH_TIME: u64 = 350;
pub const DASH_SPEED: f32 = 1.8;
pub const DASH_GETS_DANGEROUS_TIME: u64 = 150;

pub const JUMP_SPEED: f32 = 10.;

pub const INPUT_AXIS_THRESHOLD: f32 = 0.1;
pub const STAMINA_RELOAD_PER_FRAME: f32 = 1. / 60.;

pub const GAME_TIME_MS: u32 = 1000 * 60 * 2; // 2min
pub const INTRO_TIME_MS: u32 = 3000;
pub const OUTRO_TIME_MS: u32 = 5000;
pub const START_GAME_TIME_MS: u32 = 3000;

pub fn main() {
    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    // let largest = video_subsystem
    //     .displays()
    //     .iter()
    //     .max_by_key(|d| {
    //         for d2 in d.iter() {
    //             let b = d2.get_bounds().unwrap();
    //             b.w * b.h
    //         }
    //     })
    //     .unwrap();

    let mut window = video_subsystem
        .window("Arcade26", 800, 600)
        // .position_centered()
        // .fullscreen()
        .build()
        .unwrap();

    if let Some(display) = video_subsystem.displays().unwrap().get(1) {
        let bounds = display.get_bounds().unwrap();
        window.set_position(
            WindowPos::Positioned(bounds.x),
            WindowPos::Positioned(bounds.y),
        );
    }
    window.set_fullscreen(true).unwrap();

    let mut canvas = window.into_canvas();
    let creator = canvas.texture_creator();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();

    let mut sdl_eventpump = sdl_context.event_pump().unwrap();

    let mut input = ArcadeInput::new();

    let mut rendertarget = creator
        .create_texture_target(None, VIRTUAL_WIDHT, VIRTUAL_HEIGHT)
        .unwrap();
    rendertarget.set_blend_mode(BlendMode::Blend);
    rendertarget.set_scale_mode(ScaleMode::Nearest);

    let textures = Textures::new(&creator);

    // let mut current_scene = Scene::OverWorld(OverWorld::new());
    let mut current_scene = Scene::JumpGame(JumpGame::new(vec![
        Team::Blue,
        Team::Red,
        Team::Yellow,
        Team::Green,
    ]));

    let mut fps_guard = FpsGuard::new(60);

    // - Mainloop -
    loop {
        fps_guard.start_frame();
        if is_sdl_quit(&mut sdl_eventpump) {
            break;
        }
        input.update();

        if input.button_pressed(player::PlayerId(0), Button::Start) {
            println!("-ende-");
            break;
        }

        canvas.set_draw_color(Color::BLACK);
        canvas.clear();

        canvas
            .with_texture_canvas(&mut rendertarget, |mut tcnv| {
                let scene_msg;
                match &mut current_scene {
                    Scene::OverWorld(over_world) => {
                        scene_msg = over_world.update(&input);
                        over_world.draw(&mut tcnv, &textures);
                    }
                    Scene::BallGame(ball_game) => {
                        scene_msg = ball_game.update(&input);
                        ball_game.draw(&mut tcnv, &textures);
                    }
                    Scene::JumpGame(jump_game) => {
                        scene_msg = jump_game.update(&input);
                        jump_game.draw(&mut tcnv, &textures);
                    }
                    Scene::FightGame(fight_game) => {
                        scene_msg = fight_game.update(&input, fps_guard.delta_ms());
                        fight_game.draw(&mut tcnv, &textures);
                    }
                };

                match scene_msg {
                    SceneMessage::None => (),
                    SceneMessage::ChangeScene(scene) => match scene {
                        Scene::OverWorld(game_instance) => {
                            current_scene = Scene::OverWorld(game_instance);
                        }
                        Scene::JumpGame(game_instance) => {
                            current_scene = Scene::JumpGame(game_instance);
                        }
                        Scene::FightGame(game_instance) => {
                            current_scene = Scene::FightGame(game_instance);
                        }
                        Scene::BallGame(game_instance) => {
                            current_scene = Scene::BallGame(game_instance);
                        }
                    },
                }
                fps_guard.draw(tcnv, VIRTUAL_WIDHT as i32 - 20, 0);
            })
            .unwrap();
        draw_rendertarget_as_letterbox(&mut canvas, &rendertarget);

        canvas.present();
        fps_guard.end_frame();
    }
}

fn draw_rendertarget_as_letterbox(canvas: &mut WindowCanvas, render_target: &Texture<'_>) {
    // letterbox
    if let Ok(display_size) = canvas.output_size() {
        let fitted_rect = get_aspect_fitted_rect(
            VIRTUAL_WIDHT,
            VIRTUAL_HEIGHT,
            display_size.0,
            display_size.1,
        );
        canvas.copy(render_target, None, fitted_rect.rect).unwrap();
    }
}

fn is_sdl_quit(sdl_eventpump: &mut sdl3::EventPump) -> bool {
    for e in sdl_eventpump.poll_iter() {
        match e {
            sdl3::event::Event::Quit { .. } => return true,
            sdl3::event::Event::KeyDown { keycode, .. } => {
                if keycode == Some(Keycode::Escape) {
                    return true;
                }
            }
            _ => (),
        }
    }
    return false;
}

pub enum Scene {
    OverWorld(OverWorld),
    BallGame(BallGame),
    JumpGame(JumpGame),
    FightGame(FightGame),
}

#[derive(Clone, Copy)]
pub struct Arrow {
    pos: Vec2,
    direction: Vec2,
    lifetime: Instant,
    speed: f32,
    team: Team,
    colision_box: Rect,
}

impl Arrow {
    pub fn new(pos: Vec2, direction: Vec2, team: Team) -> Self {
        Self {
            pos,
            direction,
            lifetime: Instant::now() + ARROW_LIFETIME,
            speed: ARROW_SPEED,
            team,
            colision_box: Rect::new(-2, -2, 4, 4),
        }
    }
    pub fn update(&mut self) {
        self.pos = self.pos + self.direction.normalized() * self.speed
    }
    pub fn draw(&self, canvas: &mut WindowCanvas, textures: &Textures) {
        let arrow_texture = &textures.arrow;
        let q = arrow_texture.query();
        canvas
            .copy_ex(
                &textures.arrow,
                None,
                Rect::from_center(self.pos.as_point(), q.width, q.height),
                self.direction.angle() as f64,
                None,
                false,
                false,
            )
            .unwrap();
        if DEBUGMODE {
            match self.team {
                Team::Blue => {
                    canvas.set_draw_color(Color::BLUE);
                }
                Team::Red => {
                    canvas.set_draw_color(Color::RED);
                }
                Team::Yellow => {
                    canvas.set_draw_color(Color::YELLOW);
                }
                Team::Green => {
                    canvas.set_draw_color(Color::GREEN);
                }
                // Team::White => {
                //     canvas.set_draw_color(Color::WHITE);
                // }
                Team::None => (),
            }
            canvas
                .draw_rect(rect_shifted(self.colision_box, self.pos.as_point()))
                .unwrap();
            canvas.draw_point(self.pos.as_point()).unwrap();
        }
    }
    pub fn is_allive(&self) -> bool {
        Instant::now() < self.lifetime
    }
}

pub enum SceneMessage {
    None,
    ChangeScene(Scene),
}

pub struct Textures<'a> {
    pub player: Texture<'a>,
    pub overworld_background: Texture<'a>,
    pub jumpgame_background: Texture<'a>,
    pub jumpgame_paralax: Texture<'a>,
    pub fightgame_background: Texture<'a>,
    pub ballgame_background: Texture<'a>,
    pub arrow: Texture<'a>,
    pub platsch: Texture<'a>,
    pub fightgame_rules: Texture<'a>,
    pub outro_teams_red: Texture<'a>,
    pub outro_teams_blue: Texture<'a>,
}

impl<'a> Textures<'a> {
    fn new(creator: &'a TextureCreator<WindowContext>) -> Self {
        Self {
            // xxx: creator.load_texture("assets/xxx.png").unwrap(),
            outro_teams_red: creator.load_texture("assets/outro_teams_red.png").unwrap(),
            outro_teams_blue: creator.load_texture("assets/outro_teams_blue.png").unwrap(),
            player: creator.load_texture("assets/player.png").unwrap(),
            overworld_background: creator
                .load_texture("assets/overworld_background.png")
                .unwrap(),
            jumpgame_background: creator
                .load_texture("assets/jumpgame_background.png")
                .unwrap(),
            jumpgame_paralax: creator.load_texture("assets/jumpgame_paralax.png").unwrap(),
            fightgame_background: creator
                .load_texture("assets/fightgame_background.png")
                .unwrap(),
            ballgame_background: creator
                .load_texture("assets/ballgame_background.png")
                .unwrap(),
            arrow: creator.load_texture("assets/arrow.png").unwrap(),
            platsch: creator.load_texture("assets/platsch.png").unwrap(),
            fightgame_rules: creator.load_texture("assets/fightgame_rules.png").unwrap(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Team {
    None,
    Blue,
    Red,
    Green,
    Yellow,
    // White,
}

pub enum GameState {
    Intro,
    InGame,
    NextRound,
    Outro,
}

pub struct Particle {
    pos: Vec2,
    acceleration: f32,
    color: Color,
    velo: Vec2,
    lifetime: Instant,
}

impl Particle {
    pub fn new(pos: Vec2, color: Color, velo: Vec2) -> Self {
        Self {
            pos,
            acceleration: 0.15,
            color,
            velo,
            lifetime: Instant::now()
                + Duration::from_millis(random_range(0..PARTICLE_LIFETIME_MAX_MS as u64)),
        }
    }

    pub fn update(&mut self) {
        self.velo = self.velo.lerp(Vec2::zero(), self.acceleration);
        self.pos = self.pos + self.velo;
    }

    pub fn draw(&self, canvas: &mut WindowCanvas) {
        canvas.set_draw_color(self.color);
        canvas.draw_point(self.pos.as_point()).unwrap();
    }

    pub fn is_allive(&self) -> bool {
        Instant::now() < self.lifetime
    }
}
