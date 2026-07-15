use std::time::{Duration, Instant};

use sdl3::{
    pixels::Color,
    rect::{Point, Rect},
    render::WindowCanvas,
};

use crate::{
    Arrow, DEBUGMODE, Player, STUNNING_TIME, SceneMessage, Team, Textures, VIRTUAL_HEIGHT,
    arcadeinput::ArcadeInput,
    aseprite::{AnchorPosition, AsePlayer},
    math::{Vec2, rect_shifted},
    player::{PlayerMessage, Skill},
};

pub struct FightGame {
    players: Vec<Player>,
    arrows: Vec<Arrow>,
    ground_boxes: Vec<Rect>,
    platsches: Vec<Platsch>,
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
            };
            let p = Player::new(start_pos, vec![Skill::Run, Skill::Shoot, Skill::Jump], team);
            players.push(p);
        }

        let ground_boxes = vec![
            Rect::new(0, 0, 32, 180),
            Rect::new(32, 27, 260, 32),
            Rect::new(32, VIRTUAL_HEIGHT as i32 - 27 - 32, 260, 32),
            Rect::new(260, 75, 32, 30),
        ];
        Self {
            players,
            arrows: Vec::new(),
            ground_boxes,
            platsches: Vec::new(),
        }
    }

    pub fn update(&mut self, input: &ArcadeInput) -> SceneMessage {
        // - Update Players -
        for (gamepad_id, player) in self.players.iter_mut().enumerate() {
            let msg = player.update(input, gamepad_id);
            fix_player_position(player, self.ground_boxes.as_slice());
            match msg {
                PlayerMessage::ShootArrow(arrow) => self.arrows.push(arrow),
                PlayerMessage::None => (),
            }
        }

        // - Update Arrows -
        for arrow in self.arrows.iter_mut() {
            arrow.update();
        }
        self.arrows.retain(|arrow| arrow.is_allive());

        // - Update Platschs -
        for platsch in self.platsches.iter_mut() {
            platsch.update();
        }
        self.platsches
            .retain(|platsch| platsch.is_finished() == false);

        // - Check for Player/Arrow Collision -
        let mut player_stunnings = Vec::new();
        let mut arrows_to_remove = Vec::new();
        for (player_idx, player) in self.players.iter().enumerate() {
            for (arrow_idx, arrow) in self.arrows.iter().enumerate() {
                if player.team == arrow.team {
                    continue;
                }
                if player.is_dashing() || player.is_stunning() {
                    continue;
                }
                if rect_shifted(player.colision_box, player.pos.as_point())
                    .has_intersection(rect_shifted(arrow.colision_box, arrow.pos.as_point()))
                {
                    player_stunnings.push((player_idx, arrow.direction));
                    arrows_to_remove.push(arrow_idx);
                }
            }
        }
        for arrow_idx in arrows_to_remove {
            self.arrows.swap_remove(arrow_idx);
        }
        for (player_idx, stunning_direction) in player_stunnings {
            if let Some(player) = self.players.get_mut(player_idx) {
                player.stunning_end_time = Instant::now() + Duration::from_millis(STUNNING_TIME);
                player.stunning_direction = stunning_direction;
            }
        }

        // - Check for Player/Groundbox -
        let mut player_not_grounded = Vec::new();
        for (player_idx, player) in self.players.iter_mut().enumerate() {
            let mut is_player_groundet = false;

            if let Some(ground_at_current_pos) =
                ground_at_point(player.pos.as_point(), self.ground_boxes.as_slice())
            {
                is_player_groundet = true;
                player.last_ground = Some(ground_at_current_pos);
            }

            if player.is_dashing() {
                is_player_groundet = true;
                player.last_ground = None;
            }

            if is_player_groundet == false {
                player_not_grounded.push(player_idx);
            }
        }

        for player_idx in player_not_grounded {
            if let Some(player) = self.players.get_mut(player_idx) {
                self.platsches.push(Platsch::new(player.pos));
                player.pos = player.start_pos;
                player.stunning_end_time = Instant::now() + Duration::from_millis(1000);
                player.stunning_direction = Vec2::zero();
            }
        }

        SceneMessage::None
    }

    pub fn draw(&self, canvas: &mut WindowCanvas, textures: &Textures) {
        let bg = &textures.fightgame_background;
        canvas.copy(bg, None, None).unwrap();

        for player in &self.players {
            player.draw(canvas, textures);
        }
        for arrow in &self.arrows {
            arrow.draw(canvas, textures);
        }
        for platsch in &self.platsches {
            platsch.draw(canvas, textures);
        }
        if DEBUGMODE {
            canvas.set_draw_color(Color::WHITE);
            for r in &self.ground_boxes {
                canvas.draw_rect(*r).unwrap();
            }
        }
    }
}

fn fix_player_position(player: &mut Player, ground_boxes: &[Rect]) {
    if player.is_stunning() || player.is_dashing() {
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
    fn new(pos: Vec2) -> Self {
        Self {
            pos,
            ase_player: AsePlayer::from_json("assets/platsch.json"),
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
