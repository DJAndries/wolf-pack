use cubik::glium::{self, Display, Frame, Surface};
use cubik::fonts::{LoadedFont, FontText, TextAlign};
use cubik::ui::{ImageBackground, TextButton, UIError, TextInput};
use cubik::input::InputListener;
use cubik::container::RenderContainer;
use cubik::glium::glutin::event::{VirtualKeyCode, ElementState, MouseButton};
use crate::constants::APP_ID;

const NORMAL_COLOR: [f32; 4] = [0.94, 0.94, 0.94, 1.];
const HOVER_COLOR: [f32; 4] = [1., 1., 0.2, 1.];
const WHITE: [f32; 4] = [1., 1., 1., 1.];

#[derive(Copy, Clone)]
pub enum MainMenuAction {
	Start,
	Quit
}

#[derive(Clone)]
pub enum MainMenuResult {
	Start { host: String, name: String },
	Quit
}

pub struct MainMenu {
	buttons: Vec<(MainMenuAction, TextButton)>,
	btn_font: LoadedFont,
	bg: ImageBackground,
	result: Option<MainMenuResult>,
	start_dialog: StartDialog,
	info_dialog: InfoDialog
}

impl MainMenu {
	pub fn new(display: &Display) -> Result<Self, UIError> {
		Ok(Self {
			buttons: vec![
				(MainMenuAction::Start,
				 	TextButton::new("Start".to_string(), 0.15, (0., -0.3), (0.2, 0.05), NORMAL_COLOR, HOVER_COLOR)),
				(MainMenuAction::Quit,
				 	TextButton::new("Quit".to_string(), 0.15, (0., -0.5), (0.2, 0.05), NORMAL_COLOR, HOVER_COLOR))
			],
			bg: ImageBackground::new(display, "./textures/mainmenu.jpg", APP_ID, (0., 0.), (3.55, 2.))?,
			btn_font: LoadedFont::load(display, "./fonts/SourceCodePro-Light.otf", APP_ID, 80.)?,
			start_dialog: StartDialog::new(display)?,
			info_dialog: InfoDialog::new(display)?,
			result: None
		})
	}

	pub fn show_info_dialog(&mut self, content: String) {
		self.info_dialog.update_content(content);
		self.info_dialog.enabled = true;
	}

	pub fn draw(&mut self, target: &mut Frame, ctr: &RenderContainer) -> Result<Option<MainMenuResult>, UIError> {
		target.clear_color_and_depth((0., 0., 0., 1.0), 1.0); 
		self.bg.draw(target, &ctr.ui_program);
		for (_, button) in &mut self.buttons {
			button.draw(target, &ctr.display, &ctr.ui_program, &self.btn_font)?;
		}
		if self.start_dialog.enabled {
			self.result = self.start_dialog.draw(target, &ctr.display, &ctr.ui_program, &self.btn_font)?;
		}
		if self.info_dialog.enabled {
			self.info_dialog.draw(target, &ctr.display, &ctr.ui_program, &self.btn_font)?;
		}
		
		if let Some(result) = &self.result {
			let result = result.clone();
			self.result = None;
			return Ok(Some(result));
		}
		Ok(None)
	}
}

impl InputListener for MainMenu {
	fn handle_key_ev(&mut self, key: Option<VirtualKeyCode>, pressed: bool) -> bool {
		self.start_dialog.handle_key_ev(key, pressed)
	}

	fn handle_mouse_pos_ev(&mut self, pos: (f32, f32), display: &Display) -> bool {
		for (_, button) in &mut self.buttons {
			button.handle_mouse_pos_ev(pos, display);
		}
		self.start_dialog.handle_mouse_pos_ev(pos, display);
		self.info_dialog.handle_mouse_pos_ev(pos, display);
		true
	}

	fn handle_mouse_ev(&mut self, mouse_button: MouseButton, state: ElementState) -> bool {
		for (name, button) in &mut self.buttons {
			if button.handle_mouse_ev(mouse_button, state) {
				match name {
					MainMenuAction::Start => self.start_dialog.enabled = true,
					MainMenuAction::Quit => self.result = Some(MainMenuResult::Quit)
				};
				return true;
			}
		}
		if self.start_dialog.handle_mouse_ev(mouse_button, state) { return true; }
		if self.info_dialog.handle_mouse_ev(mouse_button, state) { return true; }
		false
	}

	fn handle_char_ev(&mut self, ch: char) -> bool {
		self.start_dialog.handle_char_ev(ch)
	}
}

struct StartDialog {
	bg: ImageBackground,
	host_input: TextInput,
	name_input: TextInput,
	start_btn: TextButton,
	cancel_btn: TextButton,
	title: FontText,
	name_label: FontText,
	host_label: FontText,
	enabled: bool,
	result: Option<MainMenuResult>
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

	pub fn draw(&mut self, target: &mut Frame, display: &Display, ui_program: &glium::Program, font: &LoadedFont) -> Result<Option<MainMenuResult>, UIError> {
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
			self.result = Some(MainMenuResult::Start {
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

struct InfoDialog {
	bg: ImageBackground,
	title: FontText,
	content: FontText,
	ok_btn: TextButton,
	enabled: bool
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
