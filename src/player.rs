use std::time::{Duration, Instant};

use gilrs::Button;
use sdl3::{
    pixels::Color,
    rect::{Point, Rect},
    render::WindowCanvas,
};

use crate::{
    ARROW_SPAWN_DISTANCE, ARROW_SPAWN_OFFSET_Y, Arrow, BALLGAME_BALL_X_DISTANCE_TO_WALLS,
    BALLGAME_DASH_SPEED, BALLGAME_DASH_TIME, BALLGAME_GROUND_Y, BALLGAME_JUMP_FORCE,
    BALLGAME_MAX_STAMINA, BALLGAME_PLAYER_DISTANCE_TO_WALLS, BALLGAME_PLAYER_GRAVITY_HIGH,
    BALLGAME_PLAYER_GRAVITY_LOW, BALLGAME_PLAYER_SPEED, DASH_GETS_DANGEROUS_TIME, DEBUGMODE,
    FIGHTGAME_DASH_SPEED, FIGHTGAME_DASH_TIME, FIGHTGAME_PLAYER_SPEED, FIGHTGAME_STUNNING_TIME,
    GIVE_UP_TIME_MS, INPUT_AXIS_THRESHOLD, JUMP_MAX_HOLD, JUMPGAME_GROUND_Y,
    JUMPGAME_HIGHT_GRAVITY, JUMPGAME_JUMP_FORCE, JUMPGAME_LOW_GRAVITY, JUMPGAME_PLAYER_SPEED,
    METER_RUN_SPEED, STAMINA_RELOAD_PER_FRAME, STUNNING_MOVE_FACTOR, Team, Textures,
    VIRTUAL_HEIGHT, VIRTUAL_WIDHT,
    arcadeinput::ArcadeInput,
    aseprite::{AnchorPosition, AsePlayer},
    math::{Vec2, lerp, rect_shifted},
    time::Timer,
};

pub struct Player {
    pub team: Team,
    pub start_pos: Vec2,
    pub pos: Vec2,
    pub pos_old: Vec2,
    pub velo: Vec2,
    pub fliped: bool,
    pub colision_box_small: Rect,
    pub colision_box_large: Rect,
    pub ase_player: AsePlayer,
    pub acceleration: f32,
    pub is_aiming: bool,
    pub is_jumping: bool,
    pub aim_direction: Vec2,
    pub stunned_end_time: Instant,
    pub stunning_velo: Vec2,
    pub dash_end_time: Instant,
    pub dash_direction: Vec2,
    pub last_ground: Option<Rect>,
    pub stamina: f32,
    pub state: PlayerState,
    pub jump_start_time: Instant,
    pub is_upgiving: bool,
    pub give_up_timer: Timer,
}

impl Player {
    pub fn new(pos: Vec2, team: Team) -> Self {
        let player = Player {
            pos,
            team,
            start_pos: pos,
            pos_old: pos,
            velo: Vec2::zero(),
            fliped: false,
            colision_box_small: Rect::new(-4, -14, 8, 14),
            colision_box_large: Rect::new(-7, -14, 14, 14),
            ase_player: AsePlayer::from_json("assets/player.json"),
            acceleration: 0.1,
            is_aiming: false,
            is_jumping: false,
            aim_direction: Vec2::zero(),
            stunned_end_time: Instant::now(),
            stunning_velo: Vec2::zero(),
            dash_end_time: Instant::now(),
            dash_direction: Vec2::zero(),
            last_ground: None,
            stamina: 3.,
            state: PlayerState::Idle,
            jump_start_time: Instant::now(),
            is_upgiving: false,
            give_up_timer: Timer::new(GIVE_UP_TIME_MS),
        };
        player
    }
    pub fn update_fighter(&mut self, input: &ArcadeInput, gamepad_id: usize) -> Vec<PlayerMessage> {
        let mut out = Vec::new();

        let horizontal_movement = input.axis(PlayerId(gamepad_id), gilrs::Axis::LeftStickX);
        let vertical_movement = -input.axis(PlayerId(gamepad_id), gilrs::Axis::LeftStickY);
        let input_move_direction = Vec2::new(horizontal_movement, vertical_movement).normalized();

        let horizontal_aiming = input.axis(PlayerId(gamepad_id), gilrs::Axis::RightStickX);
        let vertical_aiming = -input.axis(PlayerId(gamepad_id), gilrs::Axis::RightStickY);

        self.velo = self.velo.lerp(
            input_move_direction * FIGHTGAME_PLAYER_SPEED,
            self.acceleration,
        );

        let aiming_input = Vec2::new(horizontal_aiming, vertical_aiming).normalized();
        self.is_aiming = aiming_input != Vec2::zero();
        if self.is_aiming {
            self.aim_direction = aiming_input;
        }

        self.give_up(input, gamepad_id);

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

                // -> Stunned
                if self.stunned_end_time >= Instant::now() {
                    self.state = PlayerState::Stunned;
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

                // -> Stunned
                if self.stunned_end_time >= Instant::now() {
                    self.state = PlayerState::Stunned;
                }

                // -> Dash
                if input.just_button_pressed(PlayerId(gamepad_id), Button::South) {
                    if self.stamina >= 1. {
                        self.stamina -= 1.;
                        self.dash_direction = input_move_direction.normalized();
                        self.dash_end_time =
                            Instant::now() + Duration::from_millis(FIGHTGAME_DASH_TIME);
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
                        self.pos
                            + Vec2::new(0., -ARROW_SPAWN_OFFSET_Y)
                            + self.aim_direction * ARROW_SPAWN_DISTANCE,
                        self.aim_direction,
                        self.team,
                    );
                    out.push(PlayerMessage::ShootArrow(arrow));
                }

                // -> Idle
                if self.ase_player.is_finished() {
                    self.state = PlayerState::Idle;
                }

                // -> Stunned
                if self.stunned_end_time >= Instant::now() {
                    self.state = PlayerState::Stunned;
                }
            }
            PlayerState::Dash => {
                self.ase_player.play_tag("dash", true);

                // Velo
                self.pos = self.pos + self.dash_direction.normalized() * FIGHTGAME_DASH_SPEED;

                // -> Idle
                if self.dash_end_time < Instant::now() {
                    self.state = PlayerState::Idle;
                }
            }
            PlayerState::Stunned => {
                self.ase_player.play_tag("stunned", true);

                // Velo
                self.pos = self.pos + self.stunning_velo;

                let remaining_time = self
                    .stunned_end_time
                    .saturating_duration_since(Instant::now())
                    .as_millis() as f32;
                let stunning_progress = 1. - (1. / FIGHTGAME_STUNNING_TIME as f32 * remaining_time);
                self.stunning_velo.lerp(Vec2::zero(), stunning_progress);

                // -> Idle
                if self.stunned_end_time < Instant::now() {
                    self.state = PlayerState::Idle;
                }
            }
            _ => (),
        }

        out
    }

    fn give_up(&mut self, input: &ArcadeInput, gamepad_id: usize) {
        // - Give Up -
        if input.button_pressed(PlayerId(gamepad_id), Button::North) {
            if self.is_upgiving == false {
                self.give_up_timer.restart();
                self.is_upgiving = true;
            }
            if self.give_up_timer.is_over() {
                self.team = Team::None;
            }
        } else {
            self.is_upgiving = false;
        }
    }

    pub fn update_jumper(&mut self, input: &ArcadeInput, gamepad_id: usize) {
        let horizontal_movement = input.axis(PlayerId(gamepad_id), gilrs::Axis::LeftStickX);

        self.give_up(input, gamepad_id);

        match self.state {
            PlayerState::Move => {
                self.ase_player.play_tag("move", true);

                self.velo.x = lerp(
                    self.velo.x,
                    horizontal_movement * JUMPGAME_PLAYER_SPEED,
                    self.acceleration,
                );
                self.pos = self.pos + self.velo;

                // -> Jump
                let is_grounded = self.pos.y >= JUMPGAME_GROUND_Y as f32;
                if is_grounded {
                    self.is_jumping = false;
                }
                let is_input_jump = input.button_pressed(PlayerId(gamepad_id), Button::South);

                if is_input_jump && is_grounded {
                    self.velo.y = -JUMPGAME_JUMP_FORCE;
                    self.is_jumping = true;
                    self.jump_start_time = Instant::now();
                }

                let gravity = if is_input_jump && self.jump_start_time.elapsed() < JUMP_MAX_HOLD {
                    JUMPGAME_LOW_GRAVITY
                } else {
                    JUMPGAME_HIGHT_GRAVITY
                };
                self.velo.y += gravity;

                // -> Stunned
                if self.stunned_end_time >= Instant::now() {
                    self.state = PlayerState::Stunned;
                }
            }
            PlayerState::Stunned => {
                self.ase_player.play_tag("stunned", true);

                // Velo
                self.velo.x = -METER_RUN_SPEED * STUNNING_MOVE_FACTOR;
                self.velo.y += JUMPGAME_HIGHT_GRAVITY;
                self.pos = self.pos + self.velo;

                // -> Move
                if self.stunned_end_time < Instant::now() {
                    self.state = PlayerState::Move;
                }
            }
            _ => (),
        }

        self.pos.x = self.pos.x.clamp(0., VIRTUAL_WIDHT as f32);
        self.pos.y = self.pos.y.clamp(0., JUMPGAME_GROUND_Y as f32);
    }

    pub fn update_baller(&mut self, input: &ArcadeInput, gamepad_id: usize) {
        let horizontal_movement = input.axis(PlayerId(gamepad_id), gilrs::Axis::LeftStickX);
        let vertical_movement = input.axis(PlayerId(gamepad_id), gilrs::Axis::LeftStickY);

        self.give_up(input, gamepad_id);

        match self.state {
            PlayerState::Idle => {
                if self.is_jumping {
                    self.ase_player.play_tag("jump", true);
                } else {
                    self.ase_player.play_tag("idle", true);
                }

                // Velo
                self.velo.x = lerp(
                    self.velo.x,
                    horizontal_movement * BALLGAME_PLAYER_SPEED,
                    self.acceleration,
                );

                // Jump
                self.jumping(input, gamepad_id, horizontal_movement, vertical_movement);

                // -> Move
                if self.velo.x.abs() > INPUT_AXIS_THRESHOLD {
                    self.state = PlayerState::Move;
                }
            }
            PlayerState::Move => {
                if self.is_jumping {
                    self.ase_player.play_tag("jump", true);
                } else {
                    self.ase_player.play_tag("move", true);
                }

                // Velo
                self.velo.x = lerp(
                    self.velo.x,
                    horizontal_movement * BALLGAME_PLAYER_SPEED,
                    self.acceleration,
                );

                // Jump
                self.jumping(input, gamepad_id, horizontal_movement, vertical_movement);

                // Flip
                if self.velo.x > INPUT_AXIS_THRESHOLD {
                    self.fliped = false;
                } else if self.velo.x < -INPUT_AXIS_THRESHOLD {
                    self.fliped = true;
                }

                // -> Idle
                if self.velo.x.abs() < INPUT_AXIS_THRESHOLD {
                    self.state = PlayerState::Idle;
                }
            }
            PlayerState::Dash => {
                self.ase_player.play_tag("dash", true);

                // Velo
                self.velo = self.dash_direction.normalized() * BALLGAME_DASH_SPEED;

                // -> Idle
                if self.dash_end_time < Instant::now() {
                    self.state = PlayerState::Idle;
                }
            }
            _ => (),
        }

        self.pos = self.pos + self.velo;
        self.pos.x = self.pos.x.clamp(
            BALLGAME_PLAYER_DISTANCE_TO_WALLS,
            VIRTUAL_WIDHT as f32 - BALLGAME_PLAYER_DISTANCE_TO_WALLS,
        );
        self.pos.y = self.pos.y.clamp(0., BALLGAME_GROUND_Y as f32);
    }

    fn jumping(
        &mut self,
        input: &ArcadeInput,
        gamepad_id: usize,
        horizontal_movement: f32,
        vertical_movement: f32,
    ) {
        let is_input_jump = input.button_pressed(PlayerId(gamepad_id), Button::South);
        let is_input_just_jump = input.just_button_pressed(PlayerId(gamepad_id), Button::South);

        // -> Dash
        self.is_aiming = false;
        let input_dir = Vec2::new(horizontal_movement, -vertical_movement).normalized();
        if self.is_jumping && input_dir != Vec2::zero() {
            if self.stamina >= 1. {
                self.dash_direction = input_dir;
                self.is_aiming = true;
                self.aim_direction = input_dir;
                if is_input_just_jump {
                    self.stamina -= 1.;
                    self.dash_end_time = Instant::now() + Duration::from_millis(BALLGAME_DASH_TIME);
                    self.state = PlayerState::Dash;
                }
            }
        }

        // -> Jump
        let is_grounded = self.pos.y >= BALLGAME_GROUND_Y as f32;
        if is_grounded {
            self.is_jumping = false;
        }
        if is_grounded {
            self.stamina = BALLGAME_MAX_STAMINA;
        }

        if is_input_jump && is_grounded {
            self.velo.y = -BALLGAME_JUMP_FORCE;
            self.is_jumping = true;
            self.jump_start_time = Instant::now();
        }

        let gravity = if is_input_jump && self.jump_start_time.elapsed() < JUMP_MAX_HOLD {
            BALLGAME_PLAYER_GRAVITY_LOW
        } else {
            BALLGAME_PLAYER_GRAVITY_HIGH
        };
        self.velo.y += gravity;
    }

    fn stamina_reload(&mut self, relaxation_factor: f32) {
        self.stamina += STAMINA_RELOAD_PER_FRAME * relaxation_factor;
        self.stamina = self.stamina.clamp(0., 3.);
    }

    pub fn is_dash_dangerous(&self) -> bool {
        let dash_start = self.dash_end_time - Duration::from_millis(FIGHTGAME_DASH_TIME);
        Instant::now() > dash_start + Duration::from_millis(DASH_GETS_DANGEROUS_TIME)
    }

    pub fn draw(&self, canvas: &mut WindowCanvas, textures: &Textures, ground_y: Option<i32>) {
        // Teamfarbe
        canvas.set_draw_color(self.team.color());

        // Position
        let p = self.pos.as_point();
        let y = if let Some(gy) = ground_y { gy } else { p.y };
        canvas.draw_rect(Rect::new(p.x - 4, y - 1, 8, 2)).unwrap();
        canvas.draw_rect(Rect::new(p.x - 2, y - 2, 4, 4)).unwrap();

        // Aim
        if self.is_aiming {
            let marker_size = 16_u32;
            for i in 0..marker_size {
                if i >= ARROW_SPAWN_DISTANCE as u32 {
                    let p = (self.pos + self.aim_direction * (i * 2) as f32)
                        .as_point()
                        .offset(0, -ARROW_SPAWN_OFFSET_Y as i32);

                    canvas
                        .fill_rect(Rect::from_center(
                            p,
                            ((marker_size).saturating_sub(i)) / 2,
                            ((marker_size).saturating_sub(i)) / 2,
                        ))
                        .unwrap();
                }
            }
        }

        // Image
        self.ase_player.draw_current_frame(
            canvas,
            self.pos,
            &textures.player,
            AnchorPosition::BottomCenter,
            self.fliped,
        );

        // Stamina
        for i in 0..self.stamina as i32 {
            canvas
                .draw_point(Point::new(
                    -2 + self.pos.x as i32 + i * 2,
                    self.pos.y as i32 - 18,
                ))
                .unwrap();
        }

        // Give Up Timer
        if self.is_upgiving {
            self.give_up_timer.draw(
                canvas,
                Rect::from_center(self.pos.as_point().offset(0, -8), 16, 16),
                Color::RED,
                Color::GREEN,
            );
        }

        if DEBUGMODE {
            // Collision Box
            canvas
                .draw_rect(rect_shifted(self.colision_box_small, self.pos.as_point()))
                .unwrap();
            canvas
                .draw_rect(rect_shifted(self.colision_box_large, self.pos.as_point()))
                .unwrap();
        }
    }
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
    Jump,
}

pub fn is_only_one_team_in_game(players: &Vec<Player>) -> bool {
    let mut teams_in_game: Vec<Team> = players.iter().map(|x| x.team).collect();
    teams_in_game.retain(|xy| *xy != Team::None);
    teams_in_game.sort();
    teams_in_game.dedup();

    teams_in_game.len() <= 1
}
