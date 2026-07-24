use std::time::{Duration, Instant};

use rand::random_range;
use sdl3::{
    pixels::Color,
    rect::{Point, Rect},
    render::WindowCanvas,
};

use crate::{
    Arrow, DEBUGMODE, FIGHTGAME_STUNNING_TIME, GAME_TIME_MS, GameState, INTRO_TIME_MS,
    OUTRO_TIME_MS, Particle, Player, SCORE_RECT_HEIGHT, STUNNING_SPEED_ARROW_HIT,
    STUNNING_SPEED_DASH_HIT, Scene, SceneMessage, Team, Textures, VIRTUAL_HEIGHT, VIRTUAL_WIDHT,
    arcadeinput::ArcadeInput,
    aseprite::{AnchorPosition, AsePlayer},
    math::{Vec2, middle_direction, rect_shifted},
    overworld::OverWorld,
    player::{PlayerMessage, PlayerState},
    time::Timer,
};

pub struct FightGame {
    state: GameState,
    into_timer: Timer,
    outro_timer: Timer,
    game_timer: Timer,
    players: Vec<Player>,
    arrows: Vec<Arrow>,
    ground_boxes: Vec<Rect>,
    rule_area: Rect,
    platsches: Vec<Platsch>,
    particles: Vec<Particle>,
    ruletime_blue: u32,
    ruletime_red: u32,
    platsch_template: AsePlayer,
}

impl FightGame {
    pub fn new(player_in_game: Vec<Team>) -> Self {
        let mut players = Vec::new();
        let mut start_positions_red = vec![Vec2::new(16., 32.), Vec2::new(16., 32. + 32.)];
        let mut start_positions_blue = vec![
            Vec2::new(16., VIRTUAL_HEIGHT as f32 - 32.),
            Vec2::new(16., VIRTUAL_HEIGHT as f32 - (32. + 32.)),
        ];

        for team in player_in_game {
            let start_pos = match team {
                Team::Blue => start_positions_blue.remove(0),
                Team::Red => start_positions_red.remove(0),
                _ => Vec2::zero(),
            };
            let p = Player::new(start_pos, team);
            players.push(p);
        }

        let rule_area = Rect::new(260, 75, 32, 30);

        let ground_boxes = vec![
            Rect::new(0, 0, 32, 180),
            Rect::new(32, 27, 260, 32),
            Rect::new(32, VIRTUAL_HEIGHT as i32 - 27 - 32, 260, 32),
            Rect::new(260, 75, 32, 30),
        ];

        let platsch_json_template = AsePlayer::from_json("assets/platsch.json");

        Self {
            state: GameState::Intro,
            into_timer: Timer::new(INTRO_TIME_MS),
            game_timer: Timer::new(GAME_TIME_MS),
            outro_timer: Timer::new(OUTRO_TIME_MS),
            players,
            arrows: Vec::new(),
            rule_area,
            ground_boxes,
            platsch_template: platsch_json_template,
            platsches: Vec::new(),
            particles: Vec::new(),
            ruletime_blue: 0,
            ruletime_red: 0,
        }
    }

    pub fn update(&mut self, input: &ArcadeInput, delta_ms: u32) -> SceneMessage {
        // - Game Time -
        match self.state {
            GameState::Intro => {
                // -> InGame
                if self.into_timer.is_over() {
                    self.game_timer.restart();
                    self.state = GameState::InGame;
                }
            }
            GameState::InGame => {
                self.update_players(input);
                self.update_arrows();
                self.update_platsches();
                self.update_particles();
                self.handle_players_to_arrows();
                self.handle_players_to_dashingplayers();
                self.handle_players_to_groundboxes();
                self.handle_players_to_rulearea(delta_ms);

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
            GameState::NextRound => (),
        }

        SceneMessage::None
    }

    fn handle_players_to_rulearea(&mut self, delta_ms: u32) {
        let ruletime_rect_red = get_ruletime_rect(0, SCORE_RECT_HEIGHT as i32, self.ruletime_red);
        let ruletime_rect_blue = get_ruletime_rect(0, 0, self.ruletime_blue);
        for player in &self.players {
            let mut color = match player.team {
                Team::Blue => Color::BLUE,
                Team::Red => Color::RED,
                _ => Color::WHITE,
            };
            if random_range(0..3) == 0 {
                color = Color::WHITE;
            }
            if self.rule_area.contains_point(player.pos.as_point()) {
                match player.team {
                    Team::Blue => {
                        self.ruletime_blue += delta_ms;
                        self.particles.push(Particle::new(
                            Vec2::new(
                                ruletime_rect_blue.right() as f32,
                                ruletime_rect_blue.center().y as f32,
                            ),
                            color,
                            Vec2::random_normalized(),
                        ));
                    }
                    Team::Red => {
                        self.ruletime_red += delta_ms;
                        self.particles.push(Particle::new(
                            Vec2::new(
                                ruletime_rect_red.right() as f32,
                                ruletime_rect_red.center().y as f32,
                            ),
                            color,
                            Vec2::random_normalized(),
                        ));
                    }
                    _ => (),
                }
            }
        }
    }

    fn handle_players_to_groundboxes(&mut self) {
        let mut player_not_grounded = Vec::new();
        for (player_idx, player) in self.players.iter_mut().enumerate() {
            let mut is_player_groundet = false;

            if let Some(ground_at_current_pos) =
                ground_at_point(player.pos.as_point(), self.ground_boxes.as_slice())
            {
                is_player_groundet = true;
                player.last_ground = Some(ground_at_current_pos);
            }

            if player.state == PlayerState::Dash {
                is_player_groundet = true;
                player.last_ground = None;
            }

            if is_player_groundet == false {
                player_not_grounded.push(player_idx);
            }
        }

        for player_idx in player_not_grounded {
            if let Some(player) = self.players.get_mut(player_idx) {
                self.platsches
                    .push(Platsch::new(player.pos, &self.platsch_template));
                player.pos = player.start_pos;
                player.stunned_end_time = Instant::now() + Duration::from_millis(1000);
                player.stunning_velo = Vec2::zero();
            }
        }
    }

    fn handle_players_to_dashingplayers(&mut self) {
        let mut player_stunnings = Vec::new();
        let mut player_stop_dashing = Vec::new();
        for (player_idx, player) in self.players.iter().enumerate() {
            if player.is_dash_dangerous() == false {
                continue;
            }
            if player.state == PlayerState::Dash {
                let box1 = rect_shifted(player.colision_box_large, player.pos.as_point());
                for (player2_idx, player2) in self.players.iter().enumerate() {
                    if player_idx == player2_idx {
                        continue;
                    }
                    let box2 = rect_shifted(player2.colision_box_small, player2.pos.as_point());
                    if box1.has_intersection(box2) == false {
                        continue;
                    }
                    if player.team == player2.team {
                        continue;
                    }
                    if player2.state == PlayerState::Dash || player2.state == PlayerState::Stunned {
                        continue;
                    }
                    player_stop_dashing.push(player_idx);

                    let player_position_direction = player.pos.direction(&player2.pos).normalized();
                    let player_velo_direction = player.velo.normalized();
                    let middle_direction =
                        middle_direction(player_velo_direction, player_position_direction);
                    player_stunnings
                        .push((player2_idx, middle_direction * STUNNING_SPEED_DASH_HIT));
                }
            }
        }
        for (player_idx, stunning_velo) in player_stunnings {
            if let Some(player) = self.players.get_mut(player_idx) {
                player.stunned_end_time =
                    Instant::now() + Duration::from_millis(FIGHTGAME_STUNNING_TIME);
                player.stunning_velo = stunning_velo;
            }
        }
        for player_idx in player_stop_dashing {
            if let Some(player) = self.players.get_mut(player_idx) {
                player.state = PlayerState::Idle;
            }
        }
    }

    fn handle_players_to_arrows(&mut self) {
        let mut player_stunnings = Vec::new();
        let mut arrows_to_remove = Vec::new();
        for (player_idx, player) in self.players.iter().enumerate() {
            for (arrow_idx, arrow) in self.arrows.iter().enumerate() {
                if player.team == arrow.team {
                    continue;
                }
                if player.state == PlayerState::Dash || player.state == PlayerState::Stunned {
                    continue;
                }
                if rect_shifted(player.colision_box_small, player.pos.as_point())
                    .has_intersection(rect_shifted(arrow.colision_box, arrow.pos.as_point()))
                {
                    player_stunnings.push((
                        player_idx,
                        arrow.direction.normalized() * STUNNING_SPEED_ARROW_HIT,
                    ));
                    arrows_to_remove.push(arrow_idx);
                }
            }
        }
        for arrow_idx in arrows_to_remove {
            self.arrows.remove(arrow_idx);
        }
        for (player_idx, stunning_velo) in player_stunnings {
            if let Some(player) = self.players.get_mut(player_idx) {
                player.stunned_end_time =
                    Instant::now() + Duration::from_millis(FIGHTGAME_STUNNING_TIME);
                player.stunning_velo = stunning_velo;
            }
        }
    }

    fn update_platsches(&mut self) {
        for platsch in self.platsches.iter_mut() {
            platsch.update();
        }
        self.platsches
            .retain(|platsch| platsch.is_finished() == false);
    }

    fn update_arrows(&mut self) {
        for arrow in self.arrows.iter_mut() {
            arrow.update();
            if arrow.is_allive() == false
                && ground_at_point(arrow.pos.as_point(), &self.ground_boxes).is_none()
            {
                self.platsches
                    .push(Platsch::new(arrow.pos, &self.platsch_template));
            }
        }
        self.arrows.retain(|arrow| arrow.is_allive());
    }

    fn update_players(&mut self, input: &ArcadeInput) {
        for (gamepad_id, player) in self.players.iter_mut().enumerate() {
            if player.team == Team::None {
                continue;
            }
            let player_messages = player.update_fighter(input, gamepad_id);
            fix_player_position(player, self.ground_boxes.as_slice());
            for msg in player_messages {
                match msg {
                    PlayerMessage::ShootArrow(arrow) => self.arrows.push(arrow),
                    PlayerMessage::None => (),
                }
            }
        }
    }

    pub fn draw(&self, canvas: &mut WindowCanvas, textures: &Textures) {
        let bg = &textures.fightgame_background;
        canvas.copy(bg, None, None).unwrap();

        // Player
        for player in &self.players {
            if player.team == Team::None {
                continue;
            }
            player.draw(canvas, textures, None);
        }

        // Arrows
        for arrow in &self.arrows {
            arrow.draw(canvas, textures);
        }

        // Platsch
        for platsch in &self.platsches {
            platsch.draw(canvas, textures);
        }

        // GameTime
        self.game_timer.draw(
            canvas,
            Rect::new(0, VIRTUAL_HEIGHT as i32 - 3, VIRTUAL_WIDHT, 3),
            Color::GREEN,
            Color::RED,
        );

        // RuleTime
        canvas.set_draw_color(Color::BLUE);
        canvas
            .fill_rect(get_ruletime_rect(0, 0, self.ruletime_blue))
            .unwrap();
        canvas.set_draw_color(Color::RED);
        canvas
            .fill_rect(get_ruletime_rect(
                0,
                SCORE_RECT_HEIGHT as i32,
                self.ruletime_red,
            ))
            .unwrap();

        // Particle
        for particle in &self.particles {
            particle.draw(canvas);
        }

        // State Abhängig
        match self.state {
            GameState::Intro => {
                // - Regeln Anzeigen -
                canvas.copy(&textures.fightgame_rules, None, None).unwrap();
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
                if self.ruletime_blue > self.ruletime_red {
                    canvas.copy(&textures.outro_teams_blue, None, None).unwrap();
                } else {
                    canvas.copy(&textures.outro_teams_red, None, None).unwrap();
                }
            }
            _ => (),
        }

        // DEBUG
        if DEBUGMODE {
            canvas.set_draw_color(Color::WHITE);
            for r in &self.ground_boxes {
                canvas.draw_rect(*r).unwrap();
            }
        }
    }

    fn update_particles(&mut self) {
        for particle in self.particles.iter_mut() {
            particle.update();
        }
        self.particles.retain(|particle| particle.is_allive());
    }
}

fn get_ruletime_rect(x: i32, y: i32, rule_ms: u32) -> Rect {
    let w = VIRTUAL_WIDHT as f32 * 1. / GAME_TIME_MS as f32 * rule_ms as f32;
    Rect::new(x, y, w as u32, SCORE_RECT_HEIGHT)
}

fn fix_player_position(player: &mut Player, ground_boxes: &[Rect]) {
    if player.state == PlayerState::Stunned || player.state == PlayerState::Dash {
        return;
    }
    let new_position = player.pos.as_point();
    let Some(current_ground) = player.last_ground else {
        return;
    };
    if ground_at_point(new_position, ground_boxes).is_some() {
        return;
    }
    if new_position.x < current_ground.left() {
        player.pos.x = player.pos_old.x;
    }
    if new_position.x > current_ground.right() - 1 {
        player.pos.x = player.pos_old.x;
    }
    if new_position.y < current_ground.top() {
        player.pos.y = player.pos_old.y;
    }
    if new_position.y > current_ground.bottom() - 1 {
        player.pos.y = player.pos_old.y;
    }
}

struct Platsch {
    pos: Vec2,
    ase_player: AsePlayer,
}
impl Platsch {
    fn new(pos: Vec2, platsch_template: &AsePlayer) -> Self {
        Self {
            pos,
            ase_player: platsch_template.clone(),
            // ase_player: AsePlayer::from_json("assets/platsch.json"),
        }
    }
    fn update(&mut self) {
        self.ase_player.play_tag("default", false);
    }
    fn is_finished(&self) -> bool {
        self.ase_player.is_finished()
    }
    fn draw(&self, canvas: &mut WindowCanvas, textures: &Textures) {
        self.ase_player.draw_current_frame(
            canvas,
            self.pos,
            &textures.platsch,
            AnchorPosition::Center,
            false,
        );
    }
}

fn ground_at_point(point: Point, ground_boxes: &[Rect]) -> Option<Rect> {
    for groundbox in ground_boxes {
        if groundbox.contains_point(point) {
            return Some(*groundbox);
        }
    }
    None
}
