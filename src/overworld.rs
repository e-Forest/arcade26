use std::time::Duration;

use sdl2::{pixels::Color, rect::Rect, render::WindowCanvas};

use crate::{
    Audios, BallGame, DEBUGMODE, FightGame, IDLE_TIME_TO_SCREENSAVE_MS, JumpGame, Player, PlayerId,
    START_GAME_TIME_MS, Scene, SceneMessage, Team, Textures, VIRTUAL_HEIGHT, VIRTUAL_WIDHT,
    arcadeinput::ArcadeInput,
    aseprite::{AnchorPosition, AsePlayer},
    check_idle_timer,
    math::{Vec2, devide_rect, rect_shifted},
    screensaver::ScreenSaver,
    time::Timer,
    warn_idle_timer,
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
    idle_timer: Timer,
    chosen_scene: usize,
}

impl<'a> OverWorld {
    pub fn new() -> Self {
        let mut players = Vec::new();
        for i in 0..4 {
            let mut p = Player::new(Vec2::new(135. + (1. + i as f32) * 16., 148.), Team::None, i);
            p.fliped = true;
            players.push(p);
        }

        // - start fightgame -
        // players[0].pos = Vec2::new(80., 80.);
        // players[1].pos = Vec2::new(80., 120.);

        // - start ballgame -
        // players[0].pos = Vec2::new(240., 120.);
        // players[1].pos = Vec2::new(270., 120.);

        let ballgame_area = Rect::new(140, 95, 60, 25);
        let ballgame_devided_rect = devide_rect(ballgame_area, 1, 2);
        let fightgame_area = Rect::new(225, 118, 35, 60);
        let fightgame_devided_rect = devide_rect(fightgame_area, 2, 1);
        let jumpgame_area = Rect::new(50, 118, 70, 60);
        let jumpgame_devided_rect = devide_rect(jumpgame_area, 2, 2);

        Self {
            players,
            start_game_timer: Timer::new(START_GAME_TIME_MS),
            noplay_area: Rect::new(133, 125, 80, 40),
            jumpgame_area,
            jumpgame_area_red: jumpgame_devided_rect[0],
            jumpgame_area_blue: jumpgame_devided_rect[1],
            jumpgame_area_yellow: jumpgame_devided_rect[2],
            jumpgame_area_green: jumpgame_devided_rect[3],
            ballgame_area_blue: ballgame_devided_rect[0],
            ballgame_area_red: ballgame_devided_rect[1],
            fightgame_area_blue: fightgame_devided_rect[0],
            fightgame_area_red: fightgame_devided_rect[1],
            idle_timer: Timer::new(IDLE_TIME_TO_SCREENSAVE_MS),
            chosen_scene: 0,
        }
    }
    pub fn update(&mut self, input: &ArcadeInput, audios: &Audios) -> SceneMessage {
        if check_idle_timer(input, &mut self.idle_timer) {
            return SceneMessage::ChangeScene(Scene::ScreenSaver(ScreenSaver::new()));
        }
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
        self.update_players(input, audios);

        // - Handle Scene-Changes -
        let is_ballgame_teams_ok =
            [1, 2].contains(&players_at_ballgame_blue) && [1, 2].contains(&players_at_ballgame_red);
        let is_fightgame_teams_ok = [1, 2].contains(&players_at_fightgame_blue)
            && [1, 2].contains(&players_at_fightgame_red);
        let is_fair_temas_jumpgame = players_at_jumpgame_red <= 1
            && players_at_jumpgame_blue <= 1
            && players_at_jumpgame_yellow <= 1
            && players_at_jumpgame_green <= 1;

        if players_at_noplay <= 2 {
            if is_fair_temas_jumpgame && players_at_jumpgame + players_at_noplay == 4 {
                self.chosen_scene = 3;
                if self.start_game_timer.is_over() {
                    return SceneMessage::ChangeScene(Scene::JumpGame(JumpGame::new(
                        self.get_players_in_team(),
                    )));
                }
            } else if is_ballgame_teams_ok && players_at_ballgame + players_at_noplay == 4 {
                self.chosen_scene = 1;
                if self.start_game_timer.is_over() {
                    return SceneMessage::ChangeScene(Scene::BallGame(BallGame::new(
                        self.get_players_in_team(),
                    )));
                }
            } else if is_fightgame_teams_ok && players_at_fightgame + players_at_noplay == 4 {
                self.chosen_scene = 2;
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

    fn get_players_in_team(&self) -> Vec<(Team, usize)> {
        let mut out = Vec::new();
        for p in self.players.iter() {
            out.push((p.team, p.skin_idx));
        }
        out
    }

    fn update_players(&mut self, input: &ArcadeInput, audios: &Audios) {
        let used_skins: Vec<usize> = self.players.iter().map(|x| x.skin_idx).collect();
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
            player.update_overworlder(input, idx, audios, &used_skins);
        }
    }

    pub fn draw(&self, canvas: &mut WindowCanvas, textures: &mut Textures) {
        let background_tx = &textures.overworld_background;
        canvas.set_draw_color(Color::WHITE);
        canvas.clear();
        canvas.copy(background_tx, None, None).unwrap();

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

        // - Player anzeigen (y-ordered) -
        let mut player_y_ordered = Vec::new();

        for (idx, player) in self.players.iter().enumerate() {
            player_y_ordered.push((idx, player.pos.y as i32));
        }

        player_y_ordered.sort_by(|(_a_idx, a_y), (_b_idx, b_y)| a_y.cmp(b_y));
        for (idx, _y) in player_y_ordered {
            if let Some(player) = self.players.get(idx) {
                player.draw(canvas, textures, None, idx)
            }
        }

        // - Wenn Scenen-Wechsel-Timer läuft -
        if self.start_game_timer.remaning_time()
            < Duration::from_millis(START_GAME_TIME_MS as u64 - 100)
        {
            // - Timer anzeigen -
            self.start_game_timer.draw_as_rect(
                canvas,
                Rect::new(0, VIRTUAL_HEIGHT as i32 - 3, VIRTUAL_WIDHT, 3),
                Color::GREEN,
                Color::RED,
            );

            // - Ausgewählten Teaser anzeigen -
            // let mut teaser_x = 10;
            let teaser_x = (VIRTUAL_WIDHT - 128) / 2;
            let teaser_texture = if self.chosen_scene == 1 {
                &mut textures.ballgame_teaser
            } else if self.chosen_scene == 2 {
                &mut textures.fightgame_teaser
            } else {
                // teaser_x = VIRTUAL_WIDHT - 10 - 128;
                &mut textures.jumpgame_teaser
            };

            let alpha = 1.
                - (1. / (self.start_game_timer.wait_time().as_millis()) as f32
                    * (self.start_game_timer.remaning_time().as_millis()) as f32);
            // let alpha = 2. / (self.start_game_timer.wait_time().as_millis() / 2) as f32
            //     * (self.start_game_timer.remaning_time().as_millis() / 2) as f32;
            teaser_texture.set_alpha_mod(((alpha) * 510.).clamp(0., 255.) as u8);

            let q = teaser_texture.query();
            let (w, h) = (q.width, q.height);
            canvas
                .copy(
                    teaser_texture,
                    None,
                    Rect::new(
                        teaser_x as i32,
                        10,
                        // (VIRTUAL_WIDHT - w) as i32 / 2,
                        // (VIRTUAL_HEIGHT - h) as i32 / 2,
                        w,
                        h,
                    ),
                )
                .unwrap();
        }
        // - Alle Teaser anzeigen -
        else {
            canvas
                .copy(&textures.overworld_teasers, None, None)
                .unwrap();
        }

        warn_idle_timer(&self.idle_timer, canvas);

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
