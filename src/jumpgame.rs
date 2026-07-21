use sdl3::{rect::Rect, render::WindowCanvas};

use crate::{
    Player, Scene, SceneMessage, Team, Textures, VIRTUAL_HEIGHT, VIRTUAL_WIDHT,
    arcadeinput::ArcadeInput,
    math::Vec2,
    overworld::OverWorld,
    player::{PlayerId, PlayerState},
};

const PARALAX_FACTOR: i32 = 5;
const RUN_SPEED: f32 = 0.3;

const JUMPGAME_GROUND_Y: u32 = 120;
// const JUMPGAME_GRAVITY_ACCELERATION: f32 = 0.01;
const JUMPGAME_GRAVITY: f32 = 0.8;

pub struct JumpGame {
    players: Vec<Player>,
    meter: f32,
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

        Self { players, meter: 0. }
    }

    pub fn update(&mut self, input: &ArcadeInput) -> SceneMessage {
        self.meter -= RUN_SPEED;

        self.update_players(input);
        SceneMessage::None
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

        // Player
        for player in &self.players {
            if player.team == Team::None {
                continue;
            }
            player.draw(canvas, textures);
        }
    }

    fn update_players(&mut self, input: &ArcadeInput) {
        for (gamepad_id, player) in self.players.iter_mut().enumerate() {
            if player.team == Team::None {
                continue;
            }
            player.update_jumper(input, gamepad_id, JUMPGAME_GRAVITY, JUMPGAME_GROUND_Y);
        }
    }
}
