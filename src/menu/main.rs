use cubik::glium::{Display, Frame, Surface};
use cubik::fonts::{LoadedFont};
use cubik::ui::{ImageBackground, TextButton, UIError};
use cubik::input::InputListener;
use cubik::container::RenderContainer;
use cubik::glium::glutin::event::{VirtualKeyCode, ElementState, MouseButton};
use crate::constants::APP_ID;
use crate::menu::info::InfoDialog;
use crate::menu::start::StartDialog;
use crate::menu::settings::SettingsDialog;
use crate::settings::Settings;
use crate::menu::MenuResult;
use crate::menu::{NORMAL_COLOR, HOVER_COLOR};

#[derive(Copy, Clone)]
pub enum MainMenuAction {
	Start,
	Settings,
	Quit
}

pub struct MainMenu {
	buttons: Vec<(MainMenuAction, TextButton)>,
	btn_font: LoadedFont,
	bg: ImageBackground,
	result: Option<MenuResult>,
	start_dialog: StartDialog,
	info_dialog: InfoDialog,
	settings_dialog: SettingsDialog
}

impl MainMenu {
	pub fn new(display: &Display, settings: Settings) -> Result<Self, UIError> {
		Ok(Self {
			buttons: vec![
				(MainMenuAction::Start,
				 	TextButton::new("Start".to_string(), 0.15, (0., -0.1), (0.2, 0.05), NORMAL_COLOR, HOVER_COLOR)),
				(MainMenuAction::Settings,
				 	TextButton::new("Settings".to_string(), 0.15, (0., -0.3), (0.2, 0.05), NORMAL_COLOR, HOVER_COLOR)),
				(MainMenuAction::Quit,
				 	TextButton::new("Quit".to_string(), 0.15, (0., -0.5), (0.2, 0.05), NORMAL_COLOR, HOVER_COLOR))
			],
			bg: ImageBackground::new(display, "./textures/mainmenu.jpg", APP_ID, (0., 0.), (3.55, 2.))?,
			btn_font: LoadedFont::load(display, "./fonts/SourceCodePro-Light.otf", APP_ID, 80.)?,
			start_dialog: StartDialog::new(display)?,
			info_dialog: InfoDialog::new(display)?,
			settings_dialog: SettingsDialog::new(display, settings)?,
			result: None
		})
	}

	pub fn show_info_dialog(&mut self, content: String) {
		self.info_dialog.update_content(content);
		self.info_dialog.enabled = true;
	}

	pub fn draw(&mut self, target: &mut Frame, ctr: &RenderContainer) -> Result<Option<MenuResult>, UIError> {
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
		if self.settings_dialog.enabled {
			self.result = self.settings_dialog.draw(target, &ctr.display, &ctr.ui_program, &self.btn_font)?;
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
		
		if self.start_dialog.handle_mouse_pos_ev(pos, display) { return true; }
		if self.info_dialog.handle_mouse_pos_ev(pos, display) { return true; }
		if self.settings_dialog.handle_mouse_pos_ev(pos, display) { return true; }

		for (_, button) in &mut self.buttons {
			button.handle_mouse_pos_ev(pos, display);
		}
		true
	}

	fn handle_mouse_ev(&mut self, mouse_button: MouseButton, state: ElementState) -> bool {
		if self.start_dialog.handle_mouse_ev(mouse_button, state) { return true; }
		if self.info_dialog.handle_mouse_ev(mouse_button, state) { return true; }
		if self.settings_dialog.handle_mouse_ev(mouse_button, state) { return true; }

		for (name, button) in &mut self.buttons {
			if button.handle_mouse_ev(mouse_button, state) {
				match name {
					MainMenuAction::Start => self.start_dialog.enabled = true,
					MainMenuAction::Settings => self.settings_dialog.enabled = true,
					MainMenuAction::Quit => self.result = Some(MenuResult::Quit)
				};
				return true;
			}
		}
		
		false
	}

	fn handle_char_ev(&mut self, ch: char) -> bool {
		self.start_dialog.handle_char_ev(ch)
	}
}
