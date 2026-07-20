use sdl3::render::WindowCanvas;

use crate::{
    Player, Scene, SceneMessage, Textures,
    arcadeinput::ArcadeInput,
    math::Vec2,
    overworld::OverWorld,
    player::{PlayerId, Skill},
};

pub struct JumpGame {
    players: Vec<Player>,
}

impl JumpGame {
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
        let bg = &textures.jumpgame_background;

        canvas.copy(bg, None, None).unwrap();
    }
    // pub fn new() -> Self {
    //     let mut players = Vec::new();
    //     for i in 0..4 {
    //         let p = Player::new(
    //             Vec2::new((1. + i as f32) * 30., 145.),
    //             vec![Skill::Run, Skill::Shoot],
    //             crate::Team::Blue,
    //         );
    //         players.push(p);
    //     }
    //     Self { players }
    // }

    // pub fn update(&mut self) -> SceneMessage {
    //     SceneMessage::None
    // }
    // pub fn draw(&self, canvas: &mut WindowCanvas, textures: &Textures) {
    //     let bg = &textures.jumpgame_background;
    //     canvas.copy(bg, None, None).unwrap();
    // }
}
