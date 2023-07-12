// biblioteca
use glutin;
use gfx_window_glutin;

// caixote
use crate::{
    VoxygenErr,

    render::{
        Renderer,
        
        TgtColorFmt,
        TgtDepthFmt
    }
};

pub struct Window {
    events_loop: glutin::EventsLoop,
    window: glutin::GlWindow,
    renderer: Renderer
}

impl Window {
    pub fn new() -> Result<Window, VoxygenErr> {
        let events_loop = glutin::EventsLoop::new();

        let win_builder = glutin::WindowBuilder::new()
            .with_title("lestinia (voxygen)")
            .with_dimensions(glutin::dpi::LogicalSize::new(800.0, 500.0))
            .with_maximized(false);

        let ctx_builder = glutin::ContextBuilder::new()
            .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 2)))
            .with_sync(true);

        let (
            window,
            device,
            factory,

            tgt_color_view,
            tgt_depth_view
        ) = gfx_window_glutin::init::<TgtColorFmt, TgtDepthFmt>(
            win_builder,
            ctx_builder,

            &events_loop
        ).map_err(|err| VoxygenErr::BackendErr(Box::new(err)))?;

        let tmp = Ok(Self {
            events_loop,
            window,

            renderer: Renderer::new(
                device,
                factory,

                tgt_color_view,
                tgt_depth_view
            )?
        });

        tmp
    }

    pub fn renderer(&self) -> &Renderer { &self.renderer }
    pub fn renderer_mut(&mut self) -> &mut Renderer { &mut self.renderer }

    pub fn fetch_events(&mut self) -> Vec<Event> {
        let mut events = vec![];

        self.events_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::CloseRequested => events.push(Event::Close),
                
                _ => {}
            },

            _ => {}
        });

        events
    }

    pub fn display(&self) -> Result<(), VoxygenErr> {
        self.window
            .swap_buffers()
            .map_err(|err| VoxygenErr::BackendErr(Box::new(err)))
    }
}

pub enum Event {
    Close
}