use sdl3::{pixels::Color, rect::Rect, render::WindowCanvas};

use crate::{
    BallGame, DEBUGMODE, FightGame, JumpGame, Player, PlayerId, START_GAME_TIME_MS, Scene,
    SceneMessage, Team, Textures, VIRTUAL_HEIGHT, VIRTUAL_WIDHT,
    arcadeinput::ArcadeInput,
    aseprite::{AnchorPosition, AsePlayer},
    math::{Vec2, devide_rect, rect_shifted},
    time::Timer,
};

pub struct OverWorld {
    players: Vec<Player>,
    start_game_timer: Timer,
    noplay_area: Rect,
    jumpgame_area: Rect,
    ballgame_area_blue: Rect,
    ballgame_area_red: Rect,
    fightgame_area_blue: Rect,
    fightgame_area_red: Rect,
    jumpgame_area_red: Rect,
    jumpgame_area_blue: Rect,
    jumpgame_area_yellow: Rect,
    jumpgame_area_green: Rect,
}

impl<'a> OverWorld {
    pub fn new() -> Self {
        let mut players = Vec::new();
        for i in 0..4 {
            let mut p = Player::new(Vec2::new(230. + (1. + i as f32) * 16., 80.), Team::None);
            p.fliped = true;
            players.push(p);
        }

        let ballgame_area = Rect::new(235, 105, 70, 35);
        let ballgame_devided_rect = devide_rect(ballgame_area, 1, 2);
        let fightgame_area = Rect::new(55, 55, 35, 70);
        let fightgame_devided_rect = devide_rect(fightgame_area, 2, 1);
        let jumpgame_area = Rect::new(130, 55, 80, 60);
        let jumpgame_devided_rect = devide_rect(Rect::new(130, 55, 80, 60), 2, 2);

        Self {
            players,
            start_game_timer: Timer::new(START_GAME_TIME_MS),
            noplay_area: Rect::new(230, 55, 80, 40),
            jumpgame_area,
            jumpgame_area_red: jumpgame_devided_rect[0],
            jumpgame_area_blue: jumpgame_devided_rect[1],
            jumpgame_area_yellow: jumpgame_devided_rect[2],
            jumpgame_area_green: jumpgame_devided_rect[3],
            ballgame_area_blue: ballgame_devided_rect[0],
            ballgame_area_red: ballgame_devided_rect[1],
            fightgame_area_blue: fightgame_devided_rect[0],
            fightgame_area_red: fightgame_devided_rect[1],
        }
    }
    pub fn update(&mut self, input: &ArcadeInput) -> SceneMessage {
        // - Count Players at Areas -
        let players_at_ballgame_red = count_players_at_area(&self.players, self.ballgame_area_red);
        let players_at_ballgame_blue =
            count_players_at_area(&self.players, self.ballgame_area_blue);
        let players_at_ballgame = players_at_ballgame_blue + players_at_ballgame_red;

        let players_at_fightgame_red =
            count_players_at_area(&self.players, self.fightgame_area_red);
        let players_at_fightgame_blue =
            count_players_at_area(&self.players, self.fightgame_area_blue);
        let players_at_fightgame = players_at_fightgame_blue + players_at_fightgame_red;

        let players_at_jumpgame = count_players_at_area(&self.players, self.jumpgame_area);
        let players_at_jumpgame_red = count_players_at_area(&self.players, self.jumpgame_area_red);
        let players_at_jumpgame_blue =
            count_players_at_area(&self.players, self.jumpgame_area_blue);
        let players_at_jumpgame_yellow =
            count_players_at_area(&self.players, self.jumpgame_area_yellow);
        let players_at_jumpgame_green =
            count_players_at_area(&self.players, self.jumpgame_area_green);

        let players_at_noplay = count_players_at_area(&self.players, self.noplay_area);

        // - Update Players -
        self.update_players(input);

        // - Handle Scene-Changes -
        let is_fair_teams_ballgame = players_at_ballgame_blue == players_at_ballgame_red;
        let is_fair_teams_fightgame = players_at_fightgame_blue == players_at_fightgame_red;
        let is_fair_temas_jumpgame = players_at_jumpgame_red <= 1
            && players_at_jumpgame_blue <= 1
            && players_at_jumpgame_yellow <= 1
            && players_at_jumpgame_green <= 1;

        if players_at_noplay <= 2 {
            if is_fair_temas_jumpgame && players_at_jumpgame + players_at_noplay == 4 {
                if self.start_game_timer.is_over() {
                    return SceneMessage::ChangeScene(Scene::JumpGame(JumpGame::new(
                        self.get_players_in_team(),
                    )));
                }
            } else if is_fair_teams_ballgame && players_at_ballgame + players_at_noplay == 4 {
                if self.start_game_timer.is_over() {
                    return SceneMessage::ChangeScene(Scene::BallGame(BallGame::new()));
                }
            } else if is_fair_teams_fightgame && players_at_fightgame + players_at_noplay == 4 {
                if self.start_game_timer.is_over() {
                    return SceneMessage::ChangeScene(Scene::FightGame(FightGame::new(
                        self.get_players_in_team(),
                    )));
                }
            } else {
                self.start_game_timer.restart();
            }
        } else {
            self.start_game_timer.restart();
        }
        SceneMessage::None
    }

    fn get_players_in_team(&self) -> Vec<Team> {
        let mut out = Vec::new();
        for p in self.players.iter() {
            out.push(p.team);
        }
        out
    }

    fn update_players(&mut self, input: &ArcadeInput) {
        for (idx, player) in self.players.iter_mut().enumerate() {
            if self
                .ballgame_area_blue
                .contains_point(player.pos.as_point())
            {
                player.team = Team::Blue;
            }
            if self
                .fightgame_area_blue
                .contains_point(player.pos.as_point())
            {
                player.team = Team::Blue;
            }
            if self.ballgame_area_red.contains_point(player.pos.as_point()) {
                player.team = Team::Red;
            }
            if self
                .fightgame_area_red
                .contains_point(player.pos.as_point())
            {
                player.team = Team::Red;
            }
            if self.noplay_area.contains_point(player.pos.as_point()) {
                player.team = Team::None;
            }
            if self.jumpgame_area_red.contains_point(player.pos.as_point()) {
                player.team = Team::Red;
            }
            if self
                .jumpgame_area_blue
                .contains_point(player.pos.as_point())
            {
                player.team = Team::Blue;
            }
            if self
                .jumpgame_area_yellow
                .contains_point(player.pos.as_point())
            {
                player.team = Team::Yellow;
            }
            if self
                .jumpgame_area_green
                .contains_point(player.pos.as_point())
            {
                player.team = Team::Green;
            }
            player.update_fighter(input, idx);
        }
    }

    pub fn draw(&self, canvas: &mut WindowCanvas, textures: &Textures) {
        let background_tx = &textures.overworld_background;
        canvas.set_draw_color(Color::WHITE);
        canvas.clear();
        canvas.copy(background_tx, None, None).unwrap();

        for player in &self.players {
            player.draw(canvas, textures)
        }

        // - Areas anzeigen -
        canvas.set_draw_color(Color::WHITE);
        canvas.draw_rect(self.jumpgame_area).unwrap();
        canvas.draw_rect(self.noplay_area).unwrap();
        canvas.set_draw_color(Color::RED);
        canvas.draw_rect(self.ballgame_area_red).unwrap();
        canvas.draw_rect(self.fightgame_area_red).unwrap();
        canvas.draw_rect(self.jumpgame_area_red).unwrap();
        canvas.set_draw_color(Color::BLUE);
        canvas.draw_rect(self.ballgame_area_blue).unwrap();
        canvas.draw_rect(self.fightgame_area_blue).unwrap();
        canvas.draw_rect(self.jumpgame_area_blue).unwrap();
        canvas.set_draw_color(Color::YELLOW);
        canvas.draw_rect(self.jumpgame_area_yellow).unwrap();
        canvas.set_draw_color(Color::GREEN);
        canvas.draw_rect(self.jumpgame_area_green).unwrap();

        // - Timer anzeigen -
        self.start_game_timer.draw(
            canvas,
            Rect::new(0, VIRTUAL_HEIGHT as i32 - 3, VIRTUAL_WIDHT, 3),
            Color::GREEN,
            Color::RED,
        );

        if DEBUGMODE {}
    }
}

fn count_players_at_area(players: &Vec<Player>, area: Rect) -> usize {
    let mut out = 0;
    for p in players {
        if area.contains_point(p.pos.as_point()) {
            out += 1
        }
    }
    out
}
