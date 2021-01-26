use cubik::glium::{self, Display, Frame};
use cubik::ui::{ImageBackground, TextButton, UIError};
use cubik::fonts::{LoadedFont, FontText, TextAlign};
use cubik::glium::glutin::event::{VirtualKeyCode, ElementState, MouseButton};
use crate::menu::{NORMAL_COLOR, HOVER_COLOR};
use crate::constants::APP_ID;
use cubik::input::InputListener;

pub struct InfoDialog {
	bg: ImageBackground,
	title: FontText,
	content: FontText,
	ok_btn: TextButton,
	pub enabled: bool
}

impl InfoDialog {
	pub fn new(display: &Display) -> Result<Self, UIError> {
		Ok(Self {
			bg: ImageBackground::new(display, "./textures/dialog_lite.png", APP_ID, (0., -0.17), (1.3, 0.74))?,
			title: FontText::new("Info".to_string(), 0.07, (-0.6, 0.15), TextAlign::Left),
			content: FontText::new("".to_string(), 0.07, (0., -0.165), TextAlign::Center),
			ok_btn: TextButton::new("OK".to_string(), 0.065, (0.5, -0.44), (0.10, 0.05), NORMAL_COLOR, HOVER_COLOR),
			enabled: false
		})
	}

	pub fn update_content(&mut self, content: String) {
		self.content = FontText::new(content, 0.07, (0., -0.165), TextAlign::Center);
	}

	pub fn draw(&mut self, target: &mut Frame, display: &Display, ui_program: &glium::Program, font: &LoadedFont) -> Result<(), UIError> {
		self.bg.draw(target, ui_program);
		self.title.draw(target, display, ui_program, font)?;
		self.content.draw(target, display, ui_program, font)?;
		self.ok_btn.draw(target, display, ui_program, font)?;
		Ok(())
	}
}

impl InputListener for InfoDialog {
	fn handle_key_ev(&mut self, _key: Option<VirtualKeyCode>, _pressed: bool) -> bool {
		false
	}

	fn handle_mouse_pos_ev(&mut self, pos: (f32, f32), display: &Display) -> bool {
		if !self.enabled { return false; }
		self.ok_btn.handle_mouse_pos_ev(pos, display)
	}

	fn handle_mouse_ev(&mut self, mouse_button: MouseButton, state: ElementState) -> bool {
		if !self.enabled { return false; }
		if self.ok_btn.handle_mouse_ev(mouse_button, state) {
			self.enabled = false;
			return true;
		}
		false
	}

	fn handle_char_ev(&mut self, _ch: char) -> bool {
		false
	}
}
