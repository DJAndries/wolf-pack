use cubik::glium::{self, Display, Frame};
use cubik::glium::glutin::event::{VirtualKeyCode, ElementState, MouseButton};
use cubik::fonts::{LoadedFont, FontText, TextAlign};
use cubik::ui::{ImageBackground, TextButton, UIError, TextInput};
use cubik::input::InputListener;
use crate::menu::{NORMAL_COLOR, HOVER_COLOR, WHITE, MenuResult};
use crate::constants::APP_ID;

pub struct StartDialog {
	bg: ImageBackground,
	host_input: TextInput,
	name_input: TextInput,
	start_btn: TextButton,
	cancel_btn: TextButton,
	title: FontText,
	name_label: FontText,
	host_label: FontText,
	pub enabled: bool,
	result: Option<MenuResult>
}

impl StartDialog {
	pub fn new(display: &Display) -> Result<Self, UIError> {
		Ok(Self {
			bg: ImageBackground::new(display, "./textures/dialog.png", APP_ID, (0., -0.17), (1.0, 0.74))?,
			start_btn: TextButton::new("Join".to_string(), 0.065, (0.35, -0.44), (0.10, 0.05), NORMAL_COLOR, HOVER_COLOR),

			cancel_btn: TextButton::new("Cancel".to_string(), 0.065, (0.15, -0.44), (0.10, 0.05), NORMAL_COLOR, HOVER_COLOR),
			host_input: TextInput::new((-0.42, -0.285), (0.85, 0.08), WHITE),
			name_input: TextInput::new((-0.42, -0.085), (0.85, 0.08), WHITE),
			title: FontText::new("Join Game".to_string(), 0.07, (-0.45, 0.15), TextAlign::Left),
			name_label: FontText::new("Name:".to_string(), 0.065, (-0.44, 0.03), TextAlign::Left),
			host_label: FontText::new("Server IP Address:".to_string(), 0.065, (-0.44, -0.165), TextAlign::Left),
			enabled: false,
			result: None
		})
	}

	pub fn draw(&mut self, target: &mut Frame, display: &Display, ui_program: &glium::Program, font: &LoadedFont) -> Result<Option<MenuResult>, UIError> {
		self.bg.draw(target, ui_program);
		self.name_input.draw(target, display, ui_program, font)?;
		self.host_input.draw(target, display, ui_program, font)?;
		self.start_btn.draw(target, display, ui_program, font)?;
		self.cancel_btn.draw(target, display, ui_program, font)?;
		self.title.draw(target, display, ui_program, font)?;
		self.name_label.draw(target, display, ui_program, font)?;
		self.host_label.draw(target, display, ui_program, font)?;
		if let Some(result) = &self.result {
			let result = result.clone();
			self.result = None;
			self.enabled = false;
			return Ok(Some(result));
		}
		Ok(None)
	}
}

impl InputListener for StartDialog {
	fn handle_key_ev(&mut self, key: Option<VirtualKeyCode>, pressed: bool) -> bool {
		if !self.enabled { return false; }
		if self.host_input.handle_key_ev(key, pressed) { return true; }
		self.name_input.handle_key_ev(key, pressed)
	}

	fn handle_mouse_pos_ev(&mut self, pos: (f32, f32), display: &Display) -> bool {
		if !self.enabled { return false; }
		if self.host_input.handle_mouse_pos_ev(pos, display) { return true; }
		if self.name_input.handle_mouse_pos_ev(pos, display) { return true; }
		if self.start_btn.handle_mouse_pos_ev(pos, display) { return true; }
		self.cancel_btn.handle_mouse_pos_ev(pos, display)
	}

	fn handle_mouse_ev(&mut self, mouse_button: MouseButton, state: ElementState) -> bool {
		if !self.enabled { return false; }
		if self.host_input.handle_mouse_ev(mouse_button, state) { return true; }
		if self.name_input.handle_mouse_ev(mouse_button, state) { return true; }
		if self.start_btn.handle_mouse_ev(mouse_button, state) {
			if self.host_input.text.is_empty() || self.name_input.text.is_empty() { return true; }
			self.result = Some(MenuResult::Start {
				host: self.host_input.text.clone(),
				name: self.name_input.text.clone()
			});
			self.host_input.reset();
			self.name_input.reset();
			return true;
		}
		if self.cancel_btn.handle_mouse_ev(mouse_button, state) {
			self.host_input.reset();
			self.name_input.reset();
			self.enabled = false;
			return true;
		}
		false
	}

	fn handle_char_ev(&mut self, ch: char) -> bool {
		if !self.enabled { return false; }
		if self.host_input.handle_char_ev(ch) { return true };
		self.name_input.handle_char_ev(ch)
	}
}
