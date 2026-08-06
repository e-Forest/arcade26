use gilrs::Axis;
use sdl2::{
    pixels::Color,
    rect::Rect,
    render::{Texture, WindowCanvas},
};

use crate::{
    LOGO_BLINK_MS, OUTRO_TIME_MS, Scene, SceneMessage, Textures, VIRTUAL_HEIGHT, VIRTUAL_WIDHT,
    arcadeinput::ArcadeInput, math::Vec2, overworld::OverWorld, player::PlayerId, time::Timer,
};

pub struct ScreenSaver {
    state: ScreenSaverState,
    logo_show_timer: Timer,
    leave_screensaver: bool,
}

impl ScreenSaver {
    pub fn new() -> Self {
        Self {
            state: ScreenSaverState::BlendIn,
            logo_show_timer: Timer::new(LOGO_BLINK_MS),
            leave_screensaver: false,
        }
    }

    pub fn update(&mut self, input: &ArcadeInput) -> SceneMessage {
        if self.logo_show_timer.is_over() {
            self.logo_show_timer.restart();
            match self.state {
                ScreenSaverState::BlendIn => self.state = ScreenSaverState::BendOut,
                ScreenSaverState::BendOut => self.state = ScreenSaverState::Invisible,
                ScreenSaverState::Invisible => self.state = ScreenSaverState::BlendIn,
            }
        }
        // -> Screensaver Verlassen
        for i in 0..4 {
            if input.button_pressed(PlayerId(i), gilrs::Button::South) {
                self.leave_screensaver = true;
            }
        }
        if self.leave_screensaver {
            return SceneMessage::ChangeScene(Scene::OverWorld(OverWorld::new()));
        }

        SceneMessage::None
    }

    pub fn draw(&self, canvas: &mut WindowCanvas, logo: &mut Texture<'_>) {
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();

        // - Logo -
        let blend = 1. / self.logo_show_timer.wait_time().as_millis() as f32
            * self.logo_show_timer.remaning_time().as_millis() as f32;
        match self.state {
            ScreenSaverState::BlendIn => {
                logo.set_alpha_mod((u8::MAX as f32 * (1. - blend)) as u8);
            }
            ScreenSaverState::BendOut => {
                logo.set_alpha_mod((u8::MAX as f32 * blend) as u8);
            }
            ScreenSaverState::Invisible => {
                logo.set_alpha_mod(0);
            }
        }
        // let (w, h) = (logo.query().width, logo.query().height);
        // let dst = Rect::new(
        //     VIRTUAL_WIDHT as i32 - w as i32 - 3,
        //     VIRTUAL_HEIGHT as i32 - h as i32 - 3,
        //     w,
        //     h,
        // );
        canvas.copy(logo, None, None).unwrap();
    }
}

enum ScreenSaverState {
    BlendIn,
    BendOut,
    Invisible,
}
