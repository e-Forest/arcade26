use sdl3::{pixels::Color, rect::Rect, render::WindowCanvas};

use crate::{
    BallGame, DEBUGMODE, FightGame, JumpGame, Player, PlayerId, Scene, SceneMessage, Skill, Team,
    Textures,
    arcadeinput::ArcadeInput,
    aseprite::{AnchorPosition, AsePlayer},
    math::{Vec2, rect_shifted},
};

pub struct OverWorld {
    players: Vec<Player>,
    noplay_area: Rect,
    jumpgame_area: Rect,
    ballgame_area: Rect,
    fightgame_area: Rect,
}

impl<'a> OverWorld {
    pub fn new() -> Self {
        let mut players = Vec::new();
        for i in 0..4 {
            let p = Player::new(
                Vec2::new((1. + i as f32) * 30., 145.),
                vec![Skill::Run, Skill::Shoot],
                crate::Team::Blue,
            );
            players.push(p);
        }
        Self {
            players,
            noplay_area: Rect::new(10, 120, 140, 50),
            jumpgame_area: Rect::new(10, 10, 50, 50),
            ballgame_area: Rect::new(110, 10, 50, 50),
            fightgame_area: Rect::new(210, 10, 50, 50),
        }
    }
    pub fn update(&mut self, input: &ArcadeInput) -> SceneMessage {
        for (idx, player) in self.players.iter_mut().enumerate() {
            player.update(input, idx);
        }
        let players_at_noplay = count_players_at_area(&self.players, self.noplay_area);
        if players_at_noplay <= 2 {
            if count_players_at_area(&self.players, self.jumpgame_area) + players_at_noplay == 4 {
                return SceneMessage::ChangeScene(Scene::JumpGame(JumpGame::new()));
            }
            if count_players_at_area(&self.players, self.ballgame_area) + players_at_noplay == 4 {
                return SceneMessage::ChangeScene(Scene::BallGame(BallGame::new()));
            }
            if count_players_at_area(&self.players, self.fightgame_area) + players_at_noplay == 4 {
                return SceneMessage::ChangeScene(Scene::FightGame(FightGame::new(vec![
                    Team::Blue,
                ])));
            }
        }
        SceneMessage::None
    }

    pub fn draw(&self, canvas: &mut WindowCanvas, textures: &Textures) {
        let background_tx = &textures.overworld_background;
        canvas.set_draw_color(Color::WHITE);
        canvas.clear();
        canvas.copy(background_tx, None, None).unwrap();

        for player in &self.players {
            player.draw(canvas, textures)
        }

        if DEBUGMODE {
            canvas.set_draw_color(Color::MAGENTA);
            canvas.draw_rect(self.jumpgame_area).unwrap();
            canvas.draw_rect(self.ballgame_area).unwrap();
            canvas.draw_rect(self.fightgame_area).unwrap();
            canvas.draw_rect(self.noplay_area).unwrap();
        }
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
