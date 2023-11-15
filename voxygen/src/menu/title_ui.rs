// biblioteca
use conrod_core::{
    Positionable,
	
    Widget,
    event::Input,
    image::Id as ImgId,
	
    widget::{
        Image as ImageWidget,
        Id as WidgId
    }
};

// caixote
use crate::{
	window::Window,
    render::Renderer,
    ui::Ui
}

pub struct TitleUi {
	ui: Ui,

	widget_id: WidgId,
	title_img_id: ImgId
}

impl TitleUi {
	pub fn new(window: &mut Window) -> Self {
        let mut ui = Ui::new(window).unwrap();
		
        let widget_id = ui.id_generator().next();
        let image = image::open(concat!(env!("CARGO_MANIFEST_DIR"), "/test_assets/test.png")).unwrap();
        let title_img_id = ui.new_image(window.renderer_mut(), &image).unwrap();
		
        Self {
            ui,
			
            widget_id,
            title_img_id
        }
    }

	fn ui_layout(&mut self) {
        let mut ui_cell = self.ui.set_widgets();
		
        ImageWidget::new(self.title_img_id)
            .top_left()
            .set(self.widget_id, &mut ui_cell);
    }

	pub fn handle_event(&mut self, input: Input) {
        self.ui.handle_event(input);
    }

    pub fn maintain(&mut self, renderer: &mut Renderer) {
        self.ui_layout();
		
        self.ui.maintain(renderer);
    }

    pub fn render(&self, renderer: &mut Renderer) {
        self.ui.render(renderer);
    }
}
