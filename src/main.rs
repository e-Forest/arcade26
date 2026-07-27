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

pub const DEBUGMODE: bool = true;
pub const FIXED_FPS: u32 = 60;

pub const VIRTUAL_WIDHT: u32 = 320; // 1920/6
pub const VIRTUAL_HEIGHT: u32 = 180; // 1080/6

pub const PARTICLE_LIFETIME_MAX_MS: u32 = 600;

pub const INPUT_AXIS_THRESHOLD: f32 = 0.1;
pub const STAMINA_RELOAD_PER_FRAME: f32 = 1. / 60.;

pub const GAME_TIME_MS: u32 = 1000 * 60 * 2; // 2min
// pub const GAME_TIME_MS: u32 = 5000;
pub const INTRO_TIME_MS: u32 = 3000;
pub const OUTRO_TIME_MS: u32 = 5000;
pub const START_GAME_TIME_MS: u32 = 3000;
pub const NEXTROUND_TIME_MS: u32 = 2000;

pub const SCORE_RECT_HEIGHT: u32 = 3;

pub const GIVE_UP_TIME_MS: u32 = 3000;

// - Fightgame -
pub const FIGHTGAME_PLAYER_SPEED: f32 = 1.0;

pub const FIGHTGAME_STUNNING_TIME: u64 = 300;
pub const STUNNING_SPEED_ARROW_HIT: f32 = 1.;
pub const STUNNING_SPEED_DASH_HIT: f32 = 1.6;

pub const FIGHTGAME_DASH_TIME: u64 = 350;
pub const FIGHTGAME_DASH_SPEED: f32 = 1.8;
pub const DASH_GETS_DANGEROUS_TIME: u64 = 150;

pub const ARROW_LIFETIME: Duration = Duration::from_millis(600);
pub const ARROW_SPEED: f32 = 3.;
pub const ARROW_SPAWN_DISTANCE: f32 = 8.;
pub const ARROW_SPAWN_OFFSET_Y: f32 = 6.;

// - Jumpgame -
pub const JUMPGAME_STUNNING_LOW_POSX: f32 = 50.;
pub const JUMPGAME_STUNNING_TIME_LOW: u64 = 200;
pub const JUMPGAME_STUNNING_TIME_HIGH: u64 = 800;
pub const STUNNING_MOVE_FACTOR: f32 = 0.85;
pub const JUMPGAME_PLAYER_SPEED: f32 = 1.5;
pub const JUMPGAME_GROUND_Y: u32 = 120;

pub const JUMPGAME_JUMP_FORCE: f32 = 5.;
pub const JUMPGAME_LOW_GRAVITY: f32 = 0.2;
pub const JUMPGAME_HIGHT_GRAVITY: f32 = 0.4;
pub const JUMP_MAX_HOLD: Duration = Duration::from_millis(200);

pub const SCORE_MAX: u32 = GAME_TIME_MS / (1000 / FIXED_FPS) * VIRTUAL_WIDHT; // gemessen: 2.360.000

pub const PARALAX_FACTOR: i32 = 5;
pub const METER_RUN_SPEED: f32 = 2.;

// - Ballgame -
pub const BALLGAME_PLAYER_SPEED: f32 = 2.0;
pub const BALLGAME_GROUND_Y: u32 = 150;

pub const BALLGAME_JUMP_FORCE: f32 = 5.;
pub const BALLGAME_PLAYER_GRAVITY_LOW: f32 = 0.1;
pub const BALLGAME_PLAYER_GRAVITY_HIGH: f32 = 0.3;

pub const BALLGAME_PLAYER_DISTANCE_TO_WALLS: f32 = 30.;
pub const BALLGAME_MAX_STAMINA: f32 = 3.;

pub const BALLGAME_BALL_GRAVITY_LOW: f32 = 0.1;
pub const BALLGAME_BALL_Y_LIMIT_FOR_APPLY_HIGH_GRAVITY: f32 = 30.;
pub const BALLGAME_BALL_GRAVITY_HIGH: f32 = 0.2;
pub const BALLGAME_BALL_XBRAKE: f32 = 0.012;
pub const BALLGAME_BALL_GROUND_BOUCE_FORCE: f32 = 3.;
pub const BALLGAME_BALL_PLAYER_BOUCE_FORCE: f32 = 2.;
pub const BALLGAME_BALL_WALL_BOUCE_FORCE: f32 = 1.;
pub const BALLGAME_BALL_X_DISTANCE_TO_WALLS: f32 = 42.;
pub const BALLGAME_BALL_X_DISTANCE_TO_SCORE: f32 = BALLGAME_BALL_X_DISTANCE_TO_WALLS - 8.;
pub const BALLGAME_BALL_TIME_BETWEEN_COLLISIONS: Duration = Duration::from_millis(200);

pub const BALLGAME_RING_UPPER_EDGE: f32 = 26.;
pub const BALLGAME_RING_LOWER_EDGE: f32 = 67.;

pub const BALLGAME_PLAYER2BALL_VELO_FACTOR: f32 = 0.8;

pub const BALLGAME_DASH_TIME: u64 = 200;
pub const BALLGAME_DASH_SPEED: f32 = 3.8;

pub fn main() {
    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let mut window = video_subsystem
        .window("Arcade26", 800, 600)
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
    let mut current_scene = Scene::JumpGame(JumpGame::new(vec![Team::Yellow, Team::Green]));
    // let mut current_scene = Scene::BallGame(BallGame::new(vec![
    //     Team::Blue,
    //     Team::Red,
    //     Team::Blue,
    //     Team::Red,
    // ]));
    // let mut current_scene = Scene::FightGame(FightGame::new(vec![
    //     Team::Blue,
    //     Team::Red,
    //     Team::Blue,
    //     Team::Red,
    // ]));

    let mut fps_guard = FpsGuard::new(FIXED_FPS);

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
    pub scored_blue: Texture<'a>,
    pub scored_red: Texture<'a>,
    pub arrow: Texture<'a>,
    pub platsch: Texture<'a>,
    pub fightgame_rules: Texture<'a>,
    pub jumpgame_rules: Texture<'a>,
    pub outro_teams_red: Texture<'a>,
    pub outro_teams_blue: Texture<'a>,
    pub outro_single_red: Texture<'a>,
    pub outro_single_blue: Texture<'a>,
    pub outro_single_green: Texture<'a>,
    pub outro_single_yellow: Texture<'a>,
    pub crate_single: Texture<'a>,
    pub crate_stack: Texture<'a>,
    pub market_cart: Texture<'a>,
    pub store: Texture<'a>,
    pub ball: Texture<'a>,
}

impl<'a> Textures<'a> {
    fn new(creator: &'a TextureCreator<WindowContext>) -> Self {
        Self {
            // xxx: creator.load_texture("assets/xxx.png").unwrap(),
            scored_red: creator.load_texture("assets/scored_red.png").unwrap(),
            scored_blue: creator.load_texture("assets/scored_blue.png").unwrap(),
            ball: creator.load_texture("assets/ball.png").unwrap(),
            outro_teams_red: creator.load_texture("assets/outro_teams_red.png").unwrap(),
            outro_teams_blue: creator.load_texture("assets/outro_teams_blue.png").unwrap(),
            outro_single_red: creator.load_texture("assets/outro_single_red.png").unwrap(),
            outro_single_blue: creator
                .load_texture("assets/outro_single_blue.png")
                .unwrap(),
            outro_single_green: creator
                .load_texture("assets/outro_single_green.png")
                .unwrap(),
            outro_single_yellow: creator
                .load_texture("assets/outro_single_yellow.png")
                .unwrap(),
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
            jumpgame_rules: creator.load_texture("assets/jumpgame_rules.png").unwrap(),
            crate_single: creator.load_texture("assets/crate_single.png").unwrap(),
            crate_stack: creator.load_texture("assets/crate_stack.png").unwrap(),
            market_cart: creator.load_texture("assets/market_cart.png").unwrap(),
            store: creator.load_texture("assets/store.png").unwrap(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Team {
    None,
    Blue,
    Red,
    Green,
    Yellow,
    // White,
}
impl Team {
    pub fn color(&self) -> Color {
        match self {
            Team::Blue => Color::BLUE,
            Team::Red => Color::RED,
            Team::Yellow => Color::YELLOW,
            Team::Green => Color::GREEN,
            Team::None => Color::WHITE,
        }
    }
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
