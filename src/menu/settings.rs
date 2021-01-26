use cubik::glium::{self, Display, Frame};
use cubik::ui::{ImageBackground, TextButton, UIError, ImageButton};
use cubik::fonts::{LoadedFont, FontText, TextAlign};
use crate::constants::APP_ID;
use crate::settings::{Settings, RESOLUTION_OPTIONS};
use cubik::input::InputListener;
use cubik::glium::glutin::event::{VirtualKeyCode, ElementState, MouseButton};
use crate::menu::{NORMAL_COLOR, HOVER_COLOR, MenuResult};

pub struct SettingsDialog {
	settings: Settings,
	bg: ImageBackground,
	title: FontText,
	resolution_label: FontText,
	selected_resolution_index: usize,
	selected_resolution_label: FontText,
	res_left_select_btn: ImageButton,
	res_right_select_btn: ImageButton,
	windowed_label: FontText,
	windowed_btn: TextButton,
	apply_btn: TextButton,
	cancel_btn: TextButton,
	result: Option<MenuResult>,
	pub enabled: bool
}

impl SettingsDialog {
	pub fn new(display: &Display, settings: Settings) -> Result<Self, UIError> {
		Ok(Self {
			settings: settings,
			bg: ImageBackground::new(display, "./textures/dialog_lite.png", APP_ID, (0., 0.), (1.3, 0.84))?,
			title: FontText::new("Settings".to_string(), 0.07, (-0.6, 0.32), TextAlign::Left),
			resolution_label: FontText::new("Resolution:".to_string(), 0.07, (-0.5, 0.132), TextAlign::Left),
			selected_resolution_index: 0,
			selected_resolution_label: Self::create_resolution_label(settings.resolution),
			res_left_select_btn: ImageButton::new(
				ImageBackground::new(display, "./textures/left_arrow.png", APP_ID, (-0.5, 0.05), (0.1, 0.1))?,
				(0.15, 0.15),
				NORMAL_COLOR,
				HOVER_COLOR),
			res_right_select_btn: ImageButton::new(
				ImageBackground::new(display, "./textures/right_arrow.png", APP_ID, (0.5, 0.05), (0.1, 0.1))?,
				(0.15, 0.15),
				NORMAL_COLOR,
				HOVER_COLOR),
			windowed_label: FontText::new("Windowed:".to_string(), 0.07, (-0.5, -0.15), TextAlign::Left),
			windowed_btn: TextButton::new(if settings.windowed { "On" } else { "Off" }.to_string(),
				0.07, (0.45, -0.15), (0.1, 0.08), NORMAL_COLOR, HOVER_COLOR),
			apply_btn: TextButton::new("Apply".to_string(), 0.065, (0.5, -0.35), (0.10, 0.05), NORMAL_COLOR, HOVER_COLOR),

			cancel_btn: TextButton::new("Cancel".to_string(), 0.065, (0.25, -0.35), (0.10, 0.05), NORMAL_COLOR, HOVER_COLOR),
			enabled: false,
			result: None
		})
	}

	fn create_resolution_label(resolution: [usize; 2]) -> FontText {
		FontText::new(format!("{}x{}", resolution[0], resolution[1]), 0.07, (0., 0.05), TextAlign::Center)
	}

	pub fn draw(&mut self, target: &mut Frame, display: &Display, ui_program: &glium::Program, font: &LoadedFont) -> Result<Option<MenuResult>, UIError> {
		self.bg.draw(target, ui_program);
		self.res_left_select_btn.draw(target, ui_program);
		self.res_right_select_btn.draw(target, ui_program);
		self.title.draw(target, display, ui_program, font)?;
		self.resolution_label.draw(target, display, ui_program, font)?;
		self.selected_resolution_label.draw(target, display, ui_program, font)?;
		self.apply_btn.draw(target, display, ui_program, font)?;
		self.cancel_btn.draw(target, display, ui_program, font)?;
		self.windowed_btn.draw(target, display, ui_program, font)?;
		self.windowed_label.draw(target, display, ui_program, font)?;
		if let Some(result) = &self.result {
			let result = result.clone();
			self.result = None;
			self.enabled = false;
			return Ok(Some(result))
		}
		Ok(None)
	}
}

impl InputListener for SettingsDialog {
	fn handle_key_ev(&mut self, _key: Option<VirtualKeyCode>, _pressed: bool) -> bool {
		false
	}

	fn handle_mouse_pos_ev(&mut self, pos: (f32, f32), display: &Display) -> bool {
		if !self.enabled { return false; }
		if self.windowed_btn.handle_mouse_pos_ev(pos, display) { return true; }
		if self.apply_btn.handle_mouse_pos_ev(pos, display) { return true; }
		if self.cancel_btn.handle_mouse_pos_ev(pos, display) { return true; }
		if self.res_left_select_btn.handle_mouse_pos_ev(pos, display) { return true; }
		if self.res_right_select_btn.handle_mouse_pos_ev(pos, display) { return true; }
		true
	}

	fn handle_mouse_ev(&mut self, mouse_button: MouseButton, state: ElementState) -> bool {
		if !self.enabled { return false; }
		if self.res_left_select_btn.handle_mouse_ev(mouse_button, state) {
			self.selected_resolution_index = if self.selected_resolution_index == 0 {
				RESOLUTION_OPTIONS.len() - 1
			} else {
				self.selected_resolution_index - 1
			};
			self.settings.resolution = RESOLUTION_OPTIONS[self.selected_resolution_index];
			self.selected_resolution_label = Self::create_resolution_label(self.settings.resolution);
			return true;
		}
		if self.res_right_select_btn.handle_mouse_ev(mouse_button, state) {
			self.selected_resolution_index = (self.selected_resolution_index + 1) % RESOLUTION_OPTIONS.len();
			self.settings.resolution = RESOLUTION_OPTIONS[self.selected_resolution_index];
			self.selected_resolution_label = Self::create_resolution_label(self.settings.resolution);
			return true;
		}
		if self.apply_btn.handle_mouse_ev(mouse_button, state) {
			self.result = Some(MenuResult::SettingsChange(self.settings));
			return true;
		}
		if self.cancel_btn.handle_mouse_ev(mouse_button, state) {
			self.enabled = false;
			return true;
		}
		if self.windowed_btn.handle_mouse_ev(mouse_button, state) {
			self.settings.windowed = !self.settings.windowed;
			self.windowed_btn = TextButton::new(if self.settings.windowed { "On" } else { "Off" }.to_string(),
				0.07, (0.45, -0.15), (0.1, 0.08), NORMAL_COLOR, HOVER_COLOR);
			return true;
		}
		true
	}

	fn handle_char_ev(&mut self, _ch: char) -> bool {
		false
	}
}
