use sdl3::render::WindowCanvas;

use crate::{
    Scene, SceneMessage, Textures,
    arcadeinput::ArcadeInput,
    math::Vec2,
    overworld::OverWorld,
    player::{Player, PlayerId},
};

pub struct BallGame {
    players: Vec<Player>,
}

impl BallGame {
    pub fn new() -> Self {
        let mut players = Vec::new();
        for i in 0..4 {
            let p = Player::new(Vec2::new((1. + i as f32) * 30., 145.), crate::Team::Blue);
            players.push(p);
        }
        Self { players }
    }

    pub fn update(&mut self, input: &ArcadeInput) -> SceneMessage {
        for (gampad_id, player) in self.players.iter().enumerate() {
            if input.button_pressed(PlayerId(gampad_id), gilrs::Button::Start) {
                return SceneMessage::ChangeScene(Scene::OverWorld(OverWorld::new()));
            }
        }
        SceneMessage::None
    }

    pub fn draw(&self, canvas: &mut WindowCanvas, textures: &Textures) {
        let bg = &textures.ballgame_background;

        canvas.copy(bg, None, None).unwrap();
    }
}
