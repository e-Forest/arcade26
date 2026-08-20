use std::time::Instant;

use rand::seq::SliceRandom;
use sdl2::{
    mixer::{Chunk, Music},
    pixels::Color,
    rect::Rect,
    render::WindowCanvas,
};

use crate::{
    Audios, BALL_RADIUS, BALLGAME_BALL_GRAVITY_HIGH, BALLGAME_BALL_GRAVITY_LOW,
    BALLGAME_BALL_GROUND_BOUCE_FORCE, BALLGAME_BALL_PLAYER_BOUCE_FORCE,
    BALLGAME_BALL_WALL_BOUCE_FORCE, BALLGAME_BALL_X_DISTANCE_TO_SCORE, BALLGAME_BALL_XBRAKE,
    BALLGAME_BALL_Y_LIMIT_FOR_APPLY_HIGH_GRAVITY, BALLGAME_GROUND_Y,
    BALLGAME_PLAYER2BALL_VELO_FACTOR, BALLGAME_RING_LOWER_EDGE, BALLGAME_RING_UPPER_EDGE,
    BALLGAME_TIME_BETWEEN_BALLBOUNCE_SOUNDS, BALLGAME_WALL_X, DEBUGMODE, GAME_TIME_MS, GameState,
    IDLE_TIME_TO_SCREENSAVE_MS, INTRO_TIME_MS, NEXTROUND_TIME_MS, OUTRO_TIME_MS, PLAYER_SIZE,
    Particle, Scene, SceneMessage, Team, Textures, VIRTUAL_HEIGHT, VIRTUAL_WIDHT,
    arcadeinput::ArcadeInput,
    check_idle_timer,
    math::{Vec2, lerp, rect_shifted},
    overworld::OverWorld,
    player::{Player, PlayerId, PlayerState, is_only_one_team_in_game},
    screensaver::ScreenSaver,
    sfx_play,
    time::Timer,
    warn_idle_timer,
};

pub struct BallGame {
    players: Vec<Player>,

    into_timer: Timer,
    outro_timer: Timer,
    game_timer: Timer,
    nextround_timer: Timer,
    ball: Ball,
    particles: Vec<Particle>,
    state: GameState,
    score_red: u32,
    score_blue: u32,
    last_scored_team: Team,

    ring_area_red: Rect,
    ring_area_blue: Rect,
    idle_timer: Timer,
}

impl BallGame {
    pub fn new(player_in_game: Vec<(Team, usize)>) -> Self {
        let mut players = Vec::new();

        let mut start_positions_red = vec![Vec2::new(50., 150.), Vec2::new(60., 150.)];
        let mut start_positions_blue = vec![
            Vec2::new(VIRTUAL_WIDHT as f32 - 50., 150.),
            Vec2::new(VIRTUAL_WIDHT as f32 - 60., 150.),
        ];

        let ring_area_red = Rect::new(
            0,
            BALLGAME_RING_UPPER_EDGE as i32 + BALL_RADIUS as i32,
            BALLGAME_WALL_X as u32 + BALL_RADIUS as u32,
            BALLGAME_RING_LOWER_EDGE as u32
                - BALLGAME_RING_UPPER_EDGE as u32
                - (BALL_RADIUS * 2.) as u32,
        );

        let ring_area_blue = Rect::new(
            VIRTUAL_WIDHT as i32 - BALLGAME_WALL_X as i32 - BALL_RADIUS as i32,
            BALLGAME_RING_UPPER_EDGE as i32 + BALL_RADIUS as i32,
            BALLGAME_WALL_X as u32 + BALL_RADIUS as u32,
            BALLGAME_RING_LOWER_EDGE as u32
                - BALLGAME_RING_UPPER_EDGE as u32
                - (BALL_RADIUS * 2.) as u32,
        );

        for (_i, (team, skin_idx)) in player_in_game.iter().enumerate() {
            let start_pos = match team {
                Team::Blue => start_positions_blue.remove(0),
                Team::Red => start_positions_red.remove(0),
                _ => Vec2::zero(),
            };
            let mut p = Player::new(start_pos, *team, *skin_idx);
            p.fliped = p.team == Team::Blue;

            p.state = PlayerState::Idle;
            players.push(p);
        }

        let ball = Ball::new(Vec2::new(
            VIRTUAL_WIDHT as f32 / 2.,
            VIRTUAL_HEIGHT as f32 * 0.8,
        ));

        // - Ball fliegt in den Ring -
        // let mut ball = Ball::new(Vec2::new(VIRTUAL_WIDHT as f32 - 80., 25.));
        // ball.velo = Vec2::new(1.1, 0.6).normalized() * 3.;
        // let mut ball = Ball::new(Vec2::new(80., 25.));
        // ball.velo = Vec2::new(-1.1, 0.6).normalized() * 3.;

        Self {
            state: GameState::Intro,
            into_timer: Timer::new(INTRO_TIME_MS),
            game_timer: Timer::new(GAME_TIME_MS),
            outro_timer: Timer::new(OUTRO_TIME_MS),
            nextround_timer: Timer::new(NEXTROUND_TIME_MS),
            players,
            score_red: 0,
            score_blue: 0,
            ball,
            particles: Vec::new(),
            last_scored_team: Team::None,
            ring_area_red,
            ring_area_blue,
            idle_timer: Timer::new(IDLE_TIME_TO_SCREENSAVE_MS),
        }
    }

    pub fn update(&mut self, input: &ArcadeInput, audios: &Audios) -> SceneMessage {
        if check_idle_timer(input, &mut self.idle_timer) {
            return SceneMessage::ChangeScene(Scene::ScreenSaver(ScreenSaver::new()));
        }
        self.update_particles();

        match self.state {
            GameState::Intro => {
                // -> InGame
                if self.into_timer.is_over() {
                    self.game_timer.restart();
                    self.state = GameState::InGame;
                }
            }
            GameState::InGame => {
                self.update_players(input, audios);
                if DEBUGMODE {
                    self.ball.update(Some(input), audios);
                } else {
                    self.ball.update(None, audios);
                }
                self.handle_ball_to_rings_and_borders();

                // -> NextRound
                self.change_to_nextround(&audios);

                // -> Outro (giveup)
                if is_only_one_team_in_game(&self.players) {
                    self.outro_timer.restart();
                    self.state = GameState::Outro;
                }

                // -> Outro (timeout)
                if self.game_timer.is_over() {
                    self.outro_timer.restart();
                    Music::fade_out(1000).ok();
                    sfx_play(&audios.win_sound);
                    self.state = GameState::Outro;
                }
            }
            GameState::NextRound => {
                // -> InGame
                if self.nextround_timer.is_over() {
                    for p in self.players.iter_mut() {
                        p.pos = p.start_pos;
                        p.velo = Vec2::zero();
                        p.fliped = p.team == Team::Blue;
                    }

                    self.ball.pos = self.ball.start_pos;
                    self.ball.velo = Vec2::zero();

                    self.state = GameState::InGame;
                }
            }
            GameState::Outro => {
                // -> Scene::Overworld
                if self.outro_timer.is_over() {
                    return SceneMessage::ChangeScene(Scene::OverWorld(OverWorld::new()));
                }
            }
        }

        self.handle_players_to_ball();

        SceneMessage::None
    }

    fn handle_ball_to_rings_and_borders(&mut self) {
        let is_in_red_ring = self.ring_area_red.contains_point(self.ball.pos.as_point());
        let is_in_blue_ring = self.ring_area_blue.contains_point(self.ball.pos.as_point());

        self.ball.pos.y = self
            .ball
            .pos
            .y
            .clamp(0., BALLGAME_GROUND_Y as f32 - BALL_RADIUS);

        if self.ball.pos.y == BALLGAME_GROUND_Y as f32 - BALL_RADIUS {
            self.ball.play_bounce = true;
            self.ball.velo.y = -BALLGAME_BALL_GROUND_BOUCE_FORCE;
        }

        if self.ball.pos.x < BALLGAME_WALL_X + BALL_RADIUS {
            if !is_in_red_ring {
                self.ball.play_bounce = true;
                if self.ball.is_inring_old {
                    self.ball.velo.y = -self.ball.velo.y;
                } else {
                    self.ball.pos.x = BALLGAME_WALL_X + BALL_RADIUS;
                    self.ball.velo.x = BALLGAME_BALL_WALL_BOUCE_FORCE;
                }
            }
        }
        if self.ball.pos.x > VIRTUAL_WIDHT as f32 - BALLGAME_WALL_X - BALL_RADIUS {
            if !is_in_blue_ring {
                self.ball.play_bounce = true;
                if self.ball.is_inring_old {
                    self.ball.velo.y = -self.ball.velo.y;
                } else {
                    self.ball.pos.x = VIRTUAL_WIDHT as f32 - BALLGAME_WALL_X - BALL_RADIUS;
                    self.ball.velo.x = -BALLGAME_BALL_WALL_BOUCE_FORCE;
                }
            }
        }

        self.ball.is_inring_old = is_in_blue_ring || is_in_red_ring;
    }

    fn change_to_nextround(&mut self, audios: &Audios) {
        let add_v2;
        let is_in_red_ring = self.ring_area_red.contains_point(self.ball.pos.as_point());
        let is_in_blue_ring = self.ring_area_blue.contains_point(self.ball.pos.as_point());

        if self.ball.pos.x < BALLGAME_BALL_X_DISTANCE_TO_SCORE + BALL_RADIUS && is_in_red_ring {
            add_v2 = Vec2::new(-4., 0.);
            self.score_blue += 1;
            self.last_scored_team = Team::Blue;
        } else if self.ball.pos.x
            > VIRTUAL_WIDHT as f32 - BALLGAME_BALL_X_DISTANCE_TO_SCORE - BALL_RADIUS
            && is_in_blue_ring
        {
            add_v2 = Vec2::new(4., 0.);
            self.score_red += 1;
            self.last_scored_team = Team::Red;
        } else {
            return;
        }

        for _i in 0..120 {
            let p = Particle::new(
                self.ball.pos + Vec2::random_normalized() * 8.,
                Color::YELLOW,
                Vec2::random_normalized() * 4. + add_v2,
            );
            self.particles.push(p);
        }
        self.nextround_timer.restart();
        self.state = GameState::NextRound;
        sfx_play(&audios.scored_sound);
    }

    fn handle_players_to_ball(&mut self) {
        // if self.ball.last_collision.elapsed() < BALLGAME_BALL_TIME_BETWEEN_COLLISIONS {
        //     return;
        // }

        let mut player_indexes = Vec::new();
        for i in 0..self.players.len() {
            player_indexes.push(i);
        }
        let mut rng = rand::rng();
        player_indexes.shuffle(&mut rng);

        // for player in self.players.iter() {
        for i in player_indexes {
            let Some(player) = self.players.get(i) else {
                continue;
            };
            if player.team == Team::None {
                continue;
            }
            let player_box = rect_shifted(player.colision_box_small, player.pos.as_point());
            let ball_box = rect_shifted(self.ball.collision_box, self.ball.pos.as_point());
            let player_center = Vec2::from_point(player_box.center());
            let ball_center = Vec2::from_point(ball_box.center());

            if player_center.distance(&ball_center) < BALL_RADIUS + (PLAYER_SIZE / 2.) {
                self.ball.play_bounce = true;
                // self.ball.last_collision = Instant::now();

                // - Ball soll die geschwindigkeit des Players übernehmen -
                let (bx, by) = (self.ball.pos.x, self.ball.pos.y);
                let (px, py) = (player.pos.x, player.pos.y);
                let (pvx, pvy) = (player.velo.x, player.velo.y);
                if bx < px {
                    if pvx < 0. {
                        self.ball.velo.x = pvx;
                    }
                }
                if bx > px {
                    if pvx > 0. {
                        self.ball.velo.x = pvx;
                    }
                }
                if by > py {
                    if pvy > 0. {
                        self.ball.velo.y = pvy;
                    }
                }
                if by < py {
                    if pvy < 0. {
                        self.ball.velo.y = pvy;
                    }
                }

                // - Ball soll vom Player bouncen -
                let dir = player_center.direction(&ball_center);
                self.ball.velo = self.ball.velo + dir * BALLGAME_BALL_PLAYER_BOUCE_FORCE;
                // if player.velo.y < 0. {
                //     self.ball.velo =
                //         self.ball.velo + player.velo * BALLGAME_PLAYER2BALL_VELO_FACTOR;
                // }
            }
        }
    }

    pub fn draw(&self, canvas: &mut WindowCanvas, textures: &Textures) {
        canvas
            .copy(&textures.ballgame_background, None, None)
            .unwrap();

        // Player
        for (gamepad_id, player) in self.players.iter().enumerate() {
            if player.team == Team::None {
                continue;
            }
            player.draw(canvas, textures, Some(BALLGAME_GROUND_Y as i32), gamepad_id);
        }

        // Score
        const POKAL_W: u32 = 6;
        const POKAL_H: u32 = 8;
        for i in 0..self.score_red {
            let r = Rect::new(
                3 + i as i32 * (POKAL_W as i32 + 1),
                VIRTUAL_HEIGHT as i32 - 13,
                POKAL_W,
                POKAL_H,
            );
            canvas.set_draw_color(Color::RED);
            canvas.fill_rect(r).unwrap();
            canvas.set_draw_color(Color::YELLOW);
            canvas.draw_rect(r).unwrap();
        }
        for i in 0..self.score_blue {
            let r = Rect::new(
                VIRTUAL_WIDHT as i32 - 3 - POKAL_W as i32 - i as i32 * (POKAL_W as i32 + 1),
                VIRTUAL_HEIGHT as i32 - 13,
                POKAL_W,
                POKAL_H,
            );
            canvas.set_draw_color(Color::BLUE);
            canvas.fill_rect(r).unwrap();
            canvas.set_draw_color(Color::YELLOW);
            canvas.draw_rect(r).unwrap();
        }

        // GameTime
        self.game_timer.draw_as_rect(
            canvas,
            Rect::new(0, VIRTUAL_HEIGHT as i32 - 3, VIRTUAL_WIDHT, 3),
            Color::GREEN,
            Color::RED,
        );

        // - State Abhängig -
        match self.state {
            GameState::Intro => {
                // - Regeln Anzeigen -
                canvas.copy(&textures.jumpgame_rules, None, None).unwrap();

                // - Timer Anzeigen -
                self.into_timer.draw_as_rect(
                    canvas,
                    Rect::new(0, VIRTUAL_HEIGHT as i32 - 3, VIRTUAL_WIDHT, 3),
                    Color::GREEN,
                    Color::RED,
                );
            }
            GameState::InGame => {
                // Ball
                self.ball.draw(canvas, textures);
            }
            GameState::NextRound => {
                // - Sorer Anzeigen -
                if self.last_scored_team == Team::Blue {
                    canvas.copy(&textures.scored_blue, None, None).unwrap();
                }
                if self.last_scored_team == Team::Red {
                    canvas.copy(&textures.scored_red, None, None).unwrap();
                }

                // - Timer Anzeigen -
                self.nextround_timer.draw_as_rect(
                    canvas,
                    Rect::new(0, VIRTUAL_HEIGHT as i32 - 3, VIRTUAL_WIDHT, 3),
                    Color::GREEN,
                    Color::RED,
                );
            }
            GameState::Outro => {
                // - Timer Anzeigen -
                self.outro_timer.draw_as_rect(
                    canvas,
                    Rect::new(0, VIRTUAL_HEIGHT as i32 - 3, VIRTUAL_WIDHT, 3),
                    Color::GREEN,
                    Color::RED,
                );

                // - Gewinner Anzeigen -
                let mut winner_team = Team::None;
                if is_only_one_team_in_game(&self.players) {
                    for p in &self.players {
                        if p.team != Team::None {
                            winner_team = p.team;
                        }
                    }
                } else if self.score_blue > self.score_red {
                    winner_team = Team::Blue;
                } else if self.score_blue < self.score_red {
                    winner_team = Team::Red;
                } else {
                    winner_team = self.last_scored_team;
                }

                match winner_team {
                    Team::Blue => {
                        canvas.copy(&textures.outro_teams_blue, None, None).unwrap();
                    }
                    Team::Red => {
                        canvas.copy(&textures.outro_teams_red, None, None).unwrap();
                    }
                    _ => {
                        canvas.copy(&textures.no_winner, None, None).unwrap();
                    }
                }
            }
        }

        // Particle
        for particle in &self.particles {
            particle.draw(canvas);
        }

        warn_idle_timer(&self.idle_timer, canvas);

        // - DEBUG -
        if DEBUGMODE {
            canvas.set_draw_color(Color::MAGENTA);
            canvas.draw_rect(self.ring_area_red).unwrap();
            canvas.draw_rect(self.ring_area_blue).unwrap();
        }
    }

    fn update_players(&mut self, input: &ArcadeInput, audios: &Audios) {
        for (gamepad_id, player) in self.players.iter_mut().enumerate() {
            if player.team == Team::None {
                continue;
            }
            player.update_baller(input, gamepad_id, audios);
        }
    }

    fn update_particles(&mut self) {
        for particle in self.particles.iter_mut() {
            particle.update();
        }
        self.particles.retain(|particle| particle.is_allive());
    }
}

pub struct Ball {
    pos: Vec2,
    start_pos: Vec2,
    velo: Vec2,
    collision_box: Rect,
    // last_collision: Instant,
    is_inring_old: bool,
    play_bounce: bool,
    play_bounce_timer: Timer,
    // radius: f32,
}

impl Ball {
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            start_pos: pos,
            velo: Vec2::zero(),
            collision_box: Rect::new(-8, -8, 16, 16),
            // last_collision: Instant::now(),
            is_inring_old: false,
            play_bounce: false,
            play_bounce_timer: Timer::new(BALLGAME_TIME_BETWEEN_BALLBOUNCE_SOUNDS),
            // radius: 8.,
        }
    }
    pub fn update(&mut self, opt_input: Option<&ArcadeInput>, audios: &Audios) {
        let gravity = if self.pos.y > BALLGAME_BALL_Y_LIMIT_FOR_APPLY_HIGH_GRAVITY {
            BALLGAME_BALL_GRAVITY_LOW
        } else {
            BALLGAME_BALL_GRAVITY_HIGH
        };
        self.velo.y += gravity;
        self.velo.x = lerp(self.velo.x, 0., BALLGAME_BALL_XBRAKE);
        if let Some(input) = opt_input {
            self.velo = Vec2::new(
                input.axis(PlayerId(0), gilrs::Axis::LeftStickX),
                -input.axis(PlayerId(0), gilrs::Axis::LeftStickY),
            )
        }
        self.pos = self.pos + self.velo;
        if self.play_bounce {
            self.play_bounce = false;
            if self.play_bounce_timer.is_over() {
                sfx_play(&audios.ball_sound);
                self.play_bounce_timer.restart();
            }
        }
    }

    pub fn draw(&self, canvas: &mut WindowCanvas, textures: &Textures) {
        let p = self.pos.as_point();
        let t = &textures.ball;
        let q = t.query();
        let (w, h) = (q.width, q.height);
        let dst = Rect::new(p.x - w as i32 / 2, p.y - h as i32 / 2, w, h);
        canvas.copy(t, None, dst).unwrap();

        // - DEBUG -
        if DEBUGMODE {
            canvas.set_draw_color(Color::MAGENTA);
            canvas
                .draw_rect(rect_shifted(self.collision_box, self.pos.as_point()))
                .unwrap();
            canvas.draw_point(self.pos.as_point()).unwrap();
        }
    }
}
