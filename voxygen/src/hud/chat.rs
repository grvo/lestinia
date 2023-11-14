use crate::ui::Ui;

use conrod_core::{
	input::Key,
	position::Dimension,
	
	widget::{
		List,
		Rectangle,
		
		Text,
		TextEdit
	},

	widget_ids,

	Color,
	Colorable,
	Positionable,
	Sizeable,

	UiCell,
	Widget
};

use std::collections::VecDeque;

widget_ids! {
	struct Ids {
		message_box,
		message_box_bg,

		input,
		input_bg
	}
}

// considerar marcar isso como um widget personalizado
pub struct Chat {
	ids: Ids,
	messages: VecDeque<String>,
	input: String,
	new_messages: bool
}

impl Chat {
	pub fn new(ui: &mut Ui) -> Self {
		Chat {
			ids: Ids::new(ui.id_generator()),
			messages: VecDeque::new(),
			input: String::new(),
			new_messages: false
		}
	}

	pub fn new_message(&mut self, msg: String) {
		self.messages.push_back(msg);
		self.new_messages = true;
	}

	// determinar se a caixa de mensagem é scrollada até o bottom
	// (exemplo: o jogador está vendo novas mensagens)
	//
	// caso sim, scrollar para baixo quando novas mensagens são adicionadas
	fn scroll_new_messages(&mut self, ui_widgets: &mut UiCell) {
		if let Some(scroll) = ui_widgets
			.widget_graph()
			.widget(self.ids.message_box)
			.and_then(|widget| widget.maybe_y_scroll_state)
		{
			// caso já havia sido previamente scrollada para o bottom, manter
			if scroll.offset >= scroll.offset_bounds.start {
				ui_widgets.scroll_widget(self.ids.message_box, [0.0, std::f64::MAX]);
			}
		}
	}

	pub fn update_layout(&mut self, ui_widgets: &mut UiCell) -> Option<String> {
		// caso o enter for pressionado enviar a mensagem atual
		let new_msg = if ui_widgets
			.widget_input(self.ids.input)
			.presses()
			.key()

			.any(|key_press| match key_press.key {
				Key::Return => true,

				_ => false
			})
		{
			let new_message = self.input.clone();

			self.input.clear();
			self.new_message(new_message.clone());

			// scrollar para o bottom
			// todo: remover comentário quando obter mensagens de outras pessoas
			// ui_widgets.scroll_widget(self.ids.message_box, [0.0, std::f64::MAX]);

			Some(new_message)
		} else {
			None
		};

		// manter o scrolling
		if self.new_messages {
			self.scroll_new_messages(ui_widgets);
			self.new_messages = false;
		}

		// input do chat com retângulos como background
		let text_edit = TextEdit::new(&self.input)
			.w(500.0)
			.restrict_to_height(false)
			.font_size(30)
			.bottom_left_with_margin_on(ui_widgets.window, 10.0);

		let dims = match (
			text_edit.get_x_dimension(ui_widgets),
			text_edit.get_y_dimension(ui_widgets)
		) {
			(Dimension::Absolute(x), Dimension::Absolute(y)) => [x, y],

			_ => [0.0, 0.0]
		};

		Rectangle::fill(dims)
            .rgba(0.0, 0.0, 0.0, 0.8)
            .x_position(text_edit.get_x_position(ui_widgets))
            .y_position(text_edit.get_y_position(ui_widgets))
            .set(self.ids.input_bg, ui_widgets);
		
        if let Some(str) = text_edit.set(self.ids.input, ui_widgets) {
            self.input = str.to_string();
			
            self.input.retain(|c| c != '\n');
        }

		// caixa de mensagem
		Rectangle::fill([500.0, 300.0])
            .rgba(0.0, 0.0, 0.0, 0.5)
            .up_from(self.ids.input, 0.0)
            .set(self.ids.message_box_bg, ui_widgets);
        
		let (mut items, scrollbar) = List::flow_down(self.messages.len())
            .middle_of(self.ids.message_box_bg)
            .scrollbar_next_to()
            .scrollbar_thickness(20.0)
            .scrollbar_color(Color::Rgba(0.0, 0.0, 0.0, 1.0))
            .set(self.ids.message_box, ui_widgets);
        
		while let Some(item) = items.next(ui_widgets) {
            item.set(
                Text::new(&self.messages[item.i])
                    .font_size(30)
                    .rgba(1.0, 1.0, 1.0, 1.0),
				
                ui_widgets,
            )
        }
		
        if let Some(s) = scrollbar {
            s.set(ui_widgets)
        }

        new_msg
	}
}
