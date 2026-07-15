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
    pub stunning_end_time: Instant,
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
            stunning_end_time: Instant::now(),
            stunning_direction: Vec2::zero(),
            dash_end_time: Instant::now(),
            dash_direction: Vec2::zero(),
            last_ground: None,
            stamina: 3.,
            state: PlayerState::Idle,
        };
        player
    }
    pub fn update(&mut self, input: &ArcadeInput, gamepad_id: usize) -> PlayerMessage {
        let horizontal_movement = input.axis(PlayerId(gamepad_id), gilrs::Axis::LeftStickX);
        let vertical_movement = -input.axis(PlayerId(gamepad_id), gilrs::Axis::LeftStickY);
        let input_move_direction = Vec2::new(horizontal_movement, vertical_movement).normalized();

        let horizontal_aiming = input.axis(PlayerId(gamepad_id), gilrs::Axis::RightStickX);
        let vertical_aiming = -input.axis(PlayerId(gamepad_id), gilrs::Axis::RightStickY);

        self.velo = self
            .velo
            .lerp(input_move_direction * self.speed, self.acceleration);

        // - Schießen -
        self.aim_direction = Vec2::new(horizontal_aiming, vertical_aiming).normalized();
        self.is_aiming = self.aim_direction != Vec2::zero()
            && self.is_stunning() == false
            && self.is_dashing() == false;

        if self.is_aiming == true {
            if self.stamina >= 1. {
                // next: funktioniert nicht richtig - ich denke ich brauche eine statemachine...
                if self.ase_player.is_finished() {
                    self.stamina -= 1.;
                    let arrow = Arrow::new(
                        self.pos + self.aim_direction * ARROW_SPAWN_DISTANCE,
                        self.aim_direction,
                        self.team,
                    );

                    return PlayerMessage::ShootArrow(arrow);
                }
            }
        }

        // - Dashen -
        if input.just_button_pressed(PlayerId(gamepad_id), Button::South) {
            if self.stamina >= 1. {
                self.stamina -= 1.;
                self.dash_direction = input_move_direction.normalized();
                self.dash_end_time = Instant::now() + Duration::from_millis(DASH_TIME);
            }
        }

        // - Stamina aufladen -
        self.stamina += STAMINA_RELOAD_PER_FRAME;
        self.stamina = self.stamina.clamp(0., 3.);

        // - Animation -
        let ase_player = &mut self.ase_player;

        if self.stunning_end_time > Instant::now() {
            ase_player.play_tag("stunning", true);
        } else if self.dash_end_time > Instant::now() {
            ase_player.play_tag("dashing", true);
        } else if self.is_aiming {
            ase_player.play_tag("shoot", true);
        } else if self.velo.length() > INPUT_AXIS_THRESHOLD {
            ase_player.play_tag("run", true);
        } else {
            ase_player.play_tag("idle", true);
        }

        if self.velo.x > INPUT_AXIS_THRESHOLD {
            self.fliped = false;
        } else if self.velo.x < -INPUT_AXIS_THRESHOLD {
            self.fliped = true;
        }

        // - Velocity anwenden -
        self.pos_old = self.pos;
        if self.is_stunning() == true {
            self.pos = self.pos + self.stunning_direction.normalized() * STUNNING_SPEED;
        } else if self.is_dashing() == true {
            self.pos = self.pos + self.dash_direction.normalized() * DASH_SPEED;
        } else {
            self.pos = self.pos + self.velo;
        }

        PlayerMessage::None
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
                        self.pos.x as i32 + i * 2,
                        self.pos.y as i32 - 16,
                    ))
                    .unwrap();
            }
        }
    }
    pub fn is_stunning(&self) -> bool {
        self.stunning_end_time > Instant::now()
    }
    pub fn is_dashing(&self) -> bool {
        self.dash_end_time > Instant::now()
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
}
