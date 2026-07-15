use gilrs::{Axis, Button, Event, GamepadId, Gilrs};

use crate::player::PlayerId;

const VALUE_TO_TRUE: f32 = 0.5;

pub struct ArcadeInput {
    gilrs: Gilrs,
    player_to_gamepad: [Option<GamepadId>; 4],
    button_state: Vec<(PlayerId, Button, f32)>,
    axis_state: Vec<(PlayerId, Axis, f32)>,
    button_state_old: Vec<(PlayerId, Button, f32)>,
    axis_state_old: Vec<(PlayerId, Axis, f32)>,
}

impl ArcadeInput {
    pub fn new() -> Self {
        let gilrs = Gilrs::new().unwrap();

        let mut player_to_gamepad = [None; 4];
        for (gamepad_id, _gamepad) in gilrs.gamepads() {
            if let Some(position_of_first_none) = player_to_gamepad.iter().position(|x| x.is_none())
            {
                player_to_gamepad[position_of_first_none] = Some(gamepad_id);
                println!("new connection: {:?}", player_to_gamepad);
            }
        }

        Self {
            gilrs,
            player_to_gamepad,
            button_state: Vec::new(),
            axis_state: Vec::new(),
            button_state_old: Vec::new(),
            axis_state_old: Vec::new(),
        }
    }

    pub fn update(&mut self) {
        self.button_state_old = self.button_state.clone();
        self.axis_state_old = self.axis_state.clone();

        while let Some(Event {
            id: gamepad_id,
            event,
            ..
        }) = self.gilrs.next_event()
        {
            match event {
                gilrs::EventType::ButtonPressed(button, _code) => {
                    let value = 1.0;
                    if let Some(player_id) = self.get_player_from_gamepad_id(gamepad_id) {
                        self.button_state.push((player_id, button, value));
                    }
                }
                gilrs::EventType::ButtonReleased(button, _code) => {
                    if let Some(player_id) = self.get_player_from_gamepad_id(gamepad_id) {
                        self.button_state.retain(|(pid, btn, _val)| {
                            let player_match = *pid == player_id;
                            let button_match = *btn == button;
                            !(player_match && button_match)
                        });
                    }
                }
                gilrs::EventType::AxisChanged(axis, value, _code) => {
                    if let Some(player_id) = self.get_player_from_gamepad_id(gamepad_id) {
                        self.axis_state.retain(|(pid, ax, _val)| {
                            let player_match = *pid == player_id;
                            let axis_match = *ax == axis;
                            !(player_match && axis_match)
                        });
                        self.axis_state.push((player_id, axis, value));
                    }
                }
                gilrs::EventType::Connected => {
                    if let Some(position_of_first_none) =
                        self.player_to_gamepad.iter().position(|x| x.is_none())
                    {
                        self.player_to_gamepad[position_of_first_none] = Some(gamepad_id);
                        println!("new connection: {:?}", self.player_to_gamepad);
                    }
                }
                gilrs::EventType::Disconnected => {
                    if let Some(player_id) = self.get_player_from_gamepad_id(gamepad_id) {
                        // Player-Gamepad auf None setzen
                        self.player_to_gamepad[player_id.0] = None;
                        println!("new dis-connection: {:?}", self.player_to_gamepad);

                        // Alle Pressed Einträge entfernen
                        self.button_state.retain(|(pid, _btn, _val)| {
                            let player_match = *pid == player_id;
                            !player_match
                        });
                        println!("discon.release - {:?}", player_id);
                    }
                }
                _ => (),
            }
        }
    }

    pub fn button_pressed(&self, player_id: PlayerId, button: Button) -> bool {
        let pressed = get_button_value(&self.button_state, player_id, button) > VALUE_TO_TRUE;
        pressed
    }

    pub fn just_button_pressed(&self, player_id: PlayerId, button: Button) -> bool {
        let pressed = get_button_value(&self.button_state, player_id, button) > VALUE_TO_TRUE;
        let pressed_old =
            get_button_value(&self.button_state_old, player_id, button) > VALUE_TO_TRUE;
        pressed_old == false && pressed == true
    }

    pub fn just_button_released(&self, player_id: PlayerId, button: Button) -> bool {
        let pressed = get_button_value(&self.button_state, player_id, button) > VALUE_TO_TRUE;
        let pressed_old =
            get_button_value(&self.button_state_old, player_id, button) > VALUE_TO_TRUE;
        pressed_old == true && pressed == false
    }

    pub fn axis(&self, player_id: PlayerId, axis: Axis) -> f32 {
        get_axis_value(&self.axis_state, player_id, axis)
    }

    pub fn axis_positive(&self, player_id: PlayerId, axis: Axis) -> bool {
        get_axis_value(&self.axis_state, player_id, axis) > VALUE_TO_TRUE
    }

    pub fn axis_negative(&self, player_id: PlayerId, axis: Axis) -> bool {
        get_axis_value(&self.axis_state, player_id, axis) < -VALUE_TO_TRUE
    }

    pub fn just_axis_positive(&self, player_id: PlayerId, axis: Axis) -> bool {
        let is_positive = get_axis_value(&self.axis_state, player_id, axis) > VALUE_TO_TRUE;
        let is_positive_old = get_axis_value(&self.axis_state_old, player_id, axis) > VALUE_TO_TRUE;
        is_positive == true && is_positive_old == false
    }

    pub fn just_axis_negative(&self, player_id: PlayerId, axis: Axis) -> bool {
        let is_negative = get_axis_value(&self.axis_state, player_id, axis) < -VALUE_TO_TRUE;
        let is_negative_old =
            get_axis_value(&self.axis_state_old, player_id, axis) < -VALUE_TO_TRUE;
        is_negative == true && is_negative_old == false
    }

    pub fn get_players_gamepad_id(&self, player_id: PlayerId) -> Option<GamepadId> {
        let slot = self.player_to_gamepad.get(player_id.0)?;
        if let Some(gamepad_id) = slot {
            return Some(*gamepad_id);
        }
        None
    }

    fn get_player_from_gamepad_id(&self, gamepad_id: GamepadId) -> Option<PlayerId> {
        if let Some(position_of_gamepad_id) = self
            .player_to_gamepad
            .iter()
            .position(|x| *x == Some(gamepad_id))
        {
            return Some(PlayerId(position_of_gamepad_id));
        }
        None
    }
}

pub fn get_button_value(
    button_state: &[(PlayerId, Button, f32)],
    player_id: PlayerId,
    button: Button,
) -> f32 {
    for (pid, btn, v) in button_state {
        if *pid == player_id && *btn == button {
            return *v;
        }
    }
    0.0
}

pub fn get_axis_value(
    axis_state: &[(PlayerId, Axis, f32)],
    player_id: PlayerId,
    axis: Axis,
) -> f32 {
    for (pid, ax, v) in axis_state {
        if *pid == player_id && *ax == axis {
            return *v;
        }
    }
    0.0
}
