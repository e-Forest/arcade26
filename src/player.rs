use std::time::{Duration, Instant};

use gilrs::Button;
use sdl3::{
    pixels::Color,
    rect::{Point, Rect},
    render::WindowCanvas,
};

use crate::{
    ARROW_SPAWN_DISTANCE, Arrow, DASH_SPEED, DASH_TIME, DEBUGMODE, INPUT_AXIS_THRESHOLD,
    STAMINA_RELOAD_PER_FRAME, STUNNING_SPEED, Team, Textures,
    arcadeinput::ArcadeInput,
    aseprite::{AnchorPosition, AsePlayer},
    math::{Vec2, rect_shifted},
};

pub struct Player {
    pub team: Team,
    pub start_pos: Vec2,
    pub pos: Vec2,
    pub pos_old: Vec2,
    pub velo: Vec2,
    pub fliped: bool,
    pub colision_box: Rect,
    pub ase_player: AsePlayer,
    pub skills: Vec<Skill>,
    pub acceleration: f32,
    pub speed: f32,
    pub is_aiming: bool,
    pub aim_direction: Vec2,
    pub stunned_end_time: Instant,
    pub stunning_direction: Vec2,
    pub dash_end_time: Instant,
    pub dash_direction: Vec2,
    pub last_ground: Option<Rect>,
    pub stamina: f32,
    pub state: PlayerState,
}

impl Player {
    pub fn new(pos: Vec2, skills: Vec<Skill>, team: Team) -> Self {
        let player = Player {
            pos,
            skills,
            team,
            start_pos: pos,
            pos_old: pos,
            velo: Vec2::zero(),
            fliped: false,
            colision_box: Rect::new(-4, -4, 8, 8),
            ase_player: AsePlayer::from_json("assets/player.json"),
            acceleration: 0.1,
            speed: 1.0,
            is_aiming: false,
            aim_direction: Vec2::zero(),
            stunned_end_time: Instant::now(),
            stunning_direction: Vec2::zero(),
            dash_end_time: Instant::now(),
            dash_direction: Vec2::zero(),
            last_ground: None,
            stamina: 3.,
            state: PlayerState::Idle,
        };
        player
    }
    pub fn update(&mut self, input: &ArcadeInput, gamepad_id: usize) -> Vec<PlayerMessage> {
        let mut out = Vec::new();
        let horizontal_movement = input.axis(PlayerId(gamepad_id), gilrs::Axis::LeftStickX);
        let vertical_movement = -input.axis(PlayerId(gamepad_id), gilrs::Axis::LeftStickY);
        let input_move_direction = Vec2::new(horizontal_movement, vertical_movement).normalized();

        let horizontal_aiming = input.axis(PlayerId(gamepad_id), gilrs::Axis::RightStickX);
        let vertical_aiming = -input.axis(PlayerId(gamepad_id), gilrs::Axis::RightStickY);

        self.velo = self
            .velo
            .lerp(input_move_direction * self.speed, self.acceleration);

        let aiming_input = Vec2::new(horizontal_aiming, vertical_aiming).normalized();
        self.is_aiming = aiming_input != Vec2::zero();
        if self.is_aiming {
            self.aim_direction = aiming_input;
        }

        match self.state {
            PlayerState::Idle => {
                self.ase_player.play_tag("idle", true);

                // Stamina
                self.stamina_reload(1.);

                // -> Move
                if self.velo.length() > INPUT_AXIS_THRESHOLD {
                    self.state = PlayerState::Move;
                }

                // -> Shoot
                if self.is_aiming == true && self.stamina >= 1. {
                    self.state = PlayerState::Shoot;
                }
            }
            PlayerState::Move => {
                self.ase_player.play_tag("move", true);

                // Stamina
                self.stamina_reload(0.6);

                // Flip
                if self.velo.x > INPUT_AXIS_THRESHOLD {
                    self.fliped = false;
                } else if self.velo.x < -INPUT_AXIS_THRESHOLD {
                    self.fliped = true;
                }

                // Velo
                self.pos_old = self.pos;
                self.pos = self.pos + self.velo;

                // -> Dash
                if input.just_button_pressed(PlayerId(gamepad_id), Button::South) {
                    if self.stamina >= 1. {
                        self.stamina -= 1.;
                        self.dash_direction = input_move_direction.normalized();
                        self.dash_end_time = Instant::now() + Duration::from_millis(DASH_TIME);
                        self.state = PlayerState::Dash;
                    }
                }

                // -> Shoot
                if self.is_aiming == true && self.stamina >= 1. {
                    self.state = PlayerState::Shoot;
                }

                // -> Idle
                if self.velo.length() < INPUT_AXIS_THRESHOLD {
                    self.state = PlayerState::Idle;
                }
            }
            PlayerState::Shoot => {
                self.ase_player.play_tag("shoot", true);

                // Flip
                if self.aim_direction.x > 0. {
                    self.fliped = false;
                } else if self.aim_direction.x < 0. {
                    self.fliped = true;
                }

                // Arrow spawn
                if self.ase_player.just_frame_index(14) {
                    self.stamina -= 1.;
                    let arrow = Arrow::new(
                        self.pos + self.aim_direction * ARROW_SPAWN_DISTANCE,
                        self.aim_direction,
                        self.team,
                    );
                    out.push(PlayerMessage::ShootArrow(arrow));
                }

                // -> Idle
                if self.ase_player.is_finished() {
                    self.state = PlayerState::Idle;
                }
            }
            PlayerState::Dash => {
                self.ase_player.play_tag("dash", true);

                // Velo
                self.pos = self.pos + self.dash_direction.normalized() * DASH_SPEED;

                // -> Idle
                if self.dash_end_time < Instant::now() {
                    self.state = PlayerState::Idle;
                }
            }
            PlayerState::Stunned => {
                self.ase_player.play_tag("stunned", true);

                // Velo
                self.pos = self.pos + self.stunning_direction.normalized() * STUNNING_SPEED;

                // -> Idle
                if self.stunned_end_time < Instant::now() {
                    self.state = PlayerState::Idle;
                }
            }
        }

        out
    }

    fn stamina_reload(&mut self, relaxation_factor: f32) {
        self.stamina += STAMINA_RELOAD_PER_FRAME * relaxation_factor;
        self.stamina = self.stamina.clamp(0., 3.);
    }

    pub fn draw(&self, canvas: &mut WindowCanvas, textures: &Textures) {
        self.ase_player.draw_current_frame(
            canvas,
            self.pos,
            &textures.player,
            AnchorPosition::BottomCenter,
            self.fliped,
        );
        if DEBUGMODE {
            match self.team {
                Team::Blue => {
                    canvas.set_draw_color(Color::BLUE);
                }
                Team::Red => {
                    canvas.set_draw_color(Color::RED);
                }
            }
            canvas
                .draw_rect(rect_shifted(self.colision_box, self.pos.as_point()))
                .unwrap();
            canvas.draw_point(self.pos.as_point()).unwrap();
            canvas
                .draw_point((self.pos + self.aim_direction * ARROW_SPAWN_DISTANCE).as_point())
                .unwrap();
            for i in 0..self.stamina as i32 {
                canvas
                    .draw_point(Point::new(
                        -2 + self.pos.x as i32 + i * 2,
                        self.pos.y as i32 - 18,
                    ))
                    .unwrap();
            }
        }
    }
}

pub enum Skill {
    Run,
    Shoot,
    Jump,
    DoubleJump,
}

pub enum PlayerMessage {
    ShootArrow(Arrow),
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerState {
    Idle,
    Move,
    Shoot,
    Dash,
    Stunned,
}
