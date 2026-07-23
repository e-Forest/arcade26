use std::time::{Duration, Instant};

use rand::random_range;
use sdl3::{
    pixels::Color,
    rect::Rect,
    render::{Texture, WindowCanvas},
};

use crate::{
    DEBUGMODE, GAME_TIME_MS, GameState, INTRO_TIME_MS, JUMPGAME_GROUND_Y, JUMPGAME_STUNNING_TIME,
    METER_RUN_SPEED, OUTRO_TIME_MS, PARALAX_FACTOR, Particle, Player, SCORE_MAX, SCORE_RECT_HEIGHT,
    Scene, SceneMessage, Team, Textures, VIRTUAL_HEIGHT, VIRTUAL_WIDHT,
    arcadeinput::ArcadeInput,
    math::{Vec2, rect_shifted},
    overworld::OverWorld,
    player::{PlayerId, PlayerState},
    time::Timer,
};

const OBSTICLE_SPAWN_TIME_MS_MIN: u64 = 2000;
const OBSTICLE_SPAWN_TIME_MS_MAX: u64 = 3000;

pub struct JumpGame {
    into_timer: Timer,
    outro_timer: Timer,
    game_timer: Timer,
    players: Vec<Player>,
    obsticles: Vec<Obsticle>,
    meter: f32,
    obsticle_spawn_time: Instant,
    score: [u32; 4], // gamepad_id / score
    particles: Vec<Particle>,
    state: GameState,
}

impl JumpGame {
    pub fn new(player_in_game: Vec<Team>) -> Self {
        let mut players = Vec::new();

        for (i, team) in player_in_game.iter().enumerate() {
            let start_pos = Vec2::new(16. + i as f32 * 16., JUMPGAME_GROUND_Y as f32);
            let mut p = Player::new(start_pos, *team);
            p.state = PlayerState::Move;
            players.push(p);
        }

        Self {
            state: GameState::Intro,
            into_timer: Timer::new(INTRO_TIME_MS),
            game_timer: Timer::new(GAME_TIME_MS),
            outro_timer: Timer::new(OUTRO_TIME_MS),
            players,
            meter: 0.,
            obsticles: Vec::new(),
            obsticle_spawn_time: Instant::now(),
            score: [0; 4],
            particles: Vec::new(),
        }
    }

    pub fn update(&mut self, input: &ArcadeInput) -> SceneMessage {
        match self.state {
            GameState::Intro => {
                // -> InGame
                if self.into_timer.is_over() {
                    self.game_timer.restart();
                    self.state = GameState::InGame;
                }
            }
            GameState::InGame => {
                self.meter -= METER_RUN_SPEED;
                self.update_players(input);
                self.update_obsticles();
                self.update_particles();
                self.handle_players_to_obsitcles();
                self.handle_score();

                // -> Outro
                if self.game_timer.is_over() {
                    self.outro_timer.restart();
                    self.state = GameState::Outro;
                }
            }
            GameState::Outro => {
                // -> Scene::Overworld
                if self.outro_timer.is_over() {
                    return SceneMessage::ChangeScene(Scene::OverWorld(OverWorld::new()));
                }
            }
            _ => (),
        }

        SceneMessage::None
    }

    fn handle_score(&mut self) {
        for (gamepad_id, player) in self.players.iter().enumerate() {
            if player.team == Team::None {
                continue;
            }
            if let Some(score) = self.score.get_mut(gamepad_id) {
                *score += player.pos.x as u32;
                let score_rect =
                    get_score_rect(0, gamepad_id as i32 * SCORE_RECT_HEIGHT as i32, *score);
                let particle_color = if random_range(0..3) == 0 {
                    Color::WHITE
                } else {
                    player.team.color()
                };

                self.particles.push(Particle::new(
                    Vec2::new(score_rect.right() as f32, score_rect.center().y as f32),
                    particle_color,
                    Vec2::random_normalized(),
                ));
            }
        }
    }

    fn handle_players_to_obsitcles(&mut self) {
        for player in self.players.iter_mut() {
            let player_box = rect_shifted(player.colision_box, player.pos.as_point());
            for obsticle in self.obsticles.iter() {
                let obsticle_box = rect_shifted(obsticle.collision_box, obsticle.pos.as_point());
                if player_box.has_intersection(obsticle_box) {
                    player.stunned_end_time =
                        Instant::now() + Duration::from_millis(JUMPGAME_STUNNING_TIME);
                }
            }
        }
    }

    fn update_obsticles(&mut self) {
        for obsticle in self.obsticles.iter_mut() {
            obsticle.pos.x -= METER_RUN_SPEED;
        }

        if self.obsticle_spawn_time < Instant::now() {
            self.obsticle_spawn_time = Instant::now()
                + Duration::from_millis(random_range(
                    OBSTICLE_SPAWN_TIME_MS_MIN..OBSTICLE_SPAWN_TIME_MS_MAX,
                ));
            let obsticle_enum = match random_range(0..4) {
                0 => ObsticleEnum::Crate,
                1 => ObsticleEnum::StackOfCrates,
                2 => ObsticleEnum::MarketSign,
                _ => ObsticleEnum::MarketCart,
            };

            let obsticle = Obsticle::new(
                obsticle_enum,
                Vec2::new(VIRTUAL_WIDHT as f32, JUMPGAME_GROUND_Y as f32),
            );
            self.obsticles.push(obsticle);
        }
    }

    pub fn draw(&self, canvas: &mut WindowCanvas, textures: &Textures) {
        let paralax_widht = textures.jumpgame_paralax.width();
        let paralax_x = (self.meter / PARALAX_FACTOR as f32) as i32 % paralax_widht as i32;
        let paralax_dst_1 = Rect::new(paralax_x, 0, VIRTUAL_WIDHT, VIRTUAL_HEIGHT);
        let paralax_dst_2 = Rect::new(
            paralax_x + paralax_widht as i32,
            0,
            VIRTUAL_WIDHT,
            VIRTUAL_HEIGHT,
        );
        let background_widht = textures.jumpgame_background.width();
        let background_x = self.meter as i32 % background_widht as i32;
        let background_dst_1 = Rect::new(background_x, 0, VIRTUAL_WIDHT, VIRTUAL_HEIGHT);
        let background_dst_2 = Rect::new(
            background_x + background_widht as i32,
            0,
            VIRTUAL_WIDHT,
            VIRTUAL_HEIGHT,
        );
        canvas
            .copy(&textures.jumpgame_paralax, None, paralax_dst_1)
            .unwrap();
        canvas
            .copy(&textures.jumpgame_paralax, None, paralax_dst_2)
            .unwrap();
        canvas
            .copy(&textures.jumpgame_background, None, background_dst_1)
            .unwrap();
        canvas
            .copy(&textures.jumpgame_background, None, background_dst_2)
            .unwrap();

        // Obsticle
        for obsticle in self.obsticles.iter() {
            obsticle.draw(canvas, textures);
        }

        // Player
        for player in &self.players {
            if player.team == Team::None {
                continue;
            }
            player.draw(canvas, textures);
        }

        // GameTime
        self.game_timer.draw(
            canvas,
            Rect::new(0, VIRTUAL_HEIGHT as i32 - 3, VIRTUAL_WIDHT, 3),
            Color::GREEN,
            Color::RED,
        );

        // Score
        for (i, player) in self
            .players
            .iter()
            .filter(|x| x.team != Team::None)
            .enumerate()
        {
            let color = player.team.color();
            canvas.set_draw_color(color);
            canvas
                .fill_rect(get_score_rect(
                    0,
                    i as i32 * SCORE_RECT_HEIGHT as i32,
                    self.score[i],
                ))
                .unwrap();
        }

        // Particle
        for particle in &self.particles {
            particle.draw(canvas);
        }

        // State Abhängig
        match self.state {
            GameState::Intro => {
                // - Regeln Anzeigen -
                canvas.copy(&textures.jumpgame_rules, None, None).unwrap();
                // - Timer Anzeigen -
                self.into_timer.draw(
                    canvas,
                    Rect::new(0, VIRTUAL_HEIGHT as i32 - 3, VIRTUAL_WIDHT, 3),
                    Color::GREEN,
                    Color::RED,
                );
            }
            GameState::Outro => {
                // - Timer Anzeigen -
                self.outro_timer.draw(
                    canvas,
                    Rect::new(0, VIRTUAL_HEIGHT as i32 - 3, VIRTUAL_WIDHT, 3),
                    Color::GREEN,
                    Color::RED,
                );
                // - Gewinner Anzeigen -
                let mut max_score = 0;
                let mut winner_team = Team::None;
                for (i, score) in self.score.iter().enumerate() {
                    if *score > max_score {
                        max_score = *score;
                        if let Some(player) = self.players.get(i) {
                            winner_team = player.team;
                        }
                    }
                }
                let texture = match winner_team {
                    Team::None => &textures.outro_single_red,
                    Team::Blue => &textures.outro_single_blue,
                    Team::Red => &textures.outro_single_red,
                    Team::Green => &textures.outro_single_green,
                    Team::Yellow => &textures.outro_single_yellow,
                };
                canvas.copy(&texture, None, None).unwrap();
            }
            _ => (),
        }
    }

    fn update_players(&mut self, input: &ArcadeInput) {
        for (gamepad_id, player) in self.players.iter_mut().enumerate() {
            if player.team == Team::None {
                continue;
            }
            player.update_jumper(input, gamepad_id);
        }
    }

    fn update_particles(&mut self) {
        for particle in self.particles.iter_mut() {
            particle.update();
        }
        self.particles.retain(|particle| particle.is_allive());
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ObsticleEnum {
    Crate,
    StackOfCrates,
    MarketCart,
    MarketSign,
}

pub struct Obsticle {
    obsticle_enum: ObsticleEnum,
    pos: Vec2,
    collision_box: Rect,
}

impl Obsticle {
    pub fn new(obsticle_enum: ObsticleEnum, pos: Vec2) -> Self {
        let collision_box = match obsticle_enum {
            ObsticleEnum::Crate => Rect::new(0, 0 - 16, 16, 16),
            ObsticleEnum::StackOfCrates => Rect::new(8, 0 - 32, 16, 32),
            ObsticleEnum::MarketCart => Rect::new(0, 0 - 16, 32, 16),
            ObsticleEnum::MarketSign => Rect::new(0, 0 - 64, 16, 22),
        };
        Self {
            obsticle_enum,
            pos,
            collision_box,
        }
    }

    pub fn draw(&self, canvas: &mut WindowCanvas, textures: &Textures) {
        let texture = self.get_texture(textures);
        let (w, h) = (texture.width(), texture.height());
        let dst = Rect::new(self.pos.x as i32, self.pos.y as i32 - h as i32, w, h);
        canvas.copy(texture, None, dst).unwrap();
        if DEBUGMODE {
            canvas.set_draw_color(Color::MAGENTA);
            canvas
                .draw_rect(rect_shifted(self.collision_box, self.pos.as_point()))
                .unwrap();
        }
    }

    fn get_texture<'a>(&self, textures: &'a Textures) -> &Texture<'a> {
        match self.obsticle_enum {
            ObsticleEnum::Crate => &textures.crate_single,
            ObsticleEnum::StackOfCrates => &textures.crate_stack,
            ObsticleEnum::MarketCart => &textures.market_cart,
            ObsticleEnum::MarketSign => &textures.market_sign,
        }
    }
}

fn get_score_rect(x: i32, y: i32, score: u32) -> Rect {
    let w = VIRTUAL_WIDHT as f32 * (1. / SCORE_MAX as f32 * score as f32);
    Rect::new(x, y, w as u32, SCORE_RECT_HEIGHT)
}
