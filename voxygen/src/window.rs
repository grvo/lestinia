// biblioteca
use glutin;
use gfx_window_glutin;
use vek::*;

// caixote
use crate::{
    Error,

    render::{
        Renderer,
        
        TgtColorFmt,
        TgtDepthFmt
    }
};

pub struct Window {
    events_loop: glutin::EventsLoop,
    renderer: Renderer,
    window: glutin::GlWindow,
    cursor_grabbed: bool
}

impl Window {
    pub fn new() -> Result<Window, Error> {
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
        ).map_err(|err| Error::BackendError(Box::new(err)))?;

        let tmp = Ok(Self {
            events_loop,

            renderer: Renderer::new(
                device,
                factory,

                tgt_color_view,
                tgt_depth_view
            )?,

            window,

            cursor_grabbed: false
        });

        tmp
    }

    pub fn renderer(&self) -> &Renderer { &self.renderer }
    pub fn renderer_mut(&mut self) -> &mut Renderer { &mut self.renderer }

    pub fn fetch_events(&mut self) -> Vec<Event> {
        // copiar dados que são necessários para fechadura de eventos para isolar erros de tempo de vida
        // todo: remover isso caso/quando o compilador permitir isso
        let cursor_grabbed = self.cursor_grabbed;
        let renderer = &mut self.renderer;
        let window = &mut self.window;
        
        let mut events = vec![];

        self.events_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::CloseRequested => events.push(Event::Close),

                glutin::WindowEvent::Resized(glutin::dpi::LogicalSize {
                    width,
                    height
                }) => {
                    let (mut color_view, mut depth_view) = renderer.target_views_mut();

                    gfx_window_glutin::update_views(
                        &window,

                        &mut color_view,
                        &mut depth_view
                    );

                    events.push(Event::Resize(Vec2::new(width as u32, height as u32)));
                },
                
                glutin::WindowEvent::ReceivedCharacter(c) => events.push(Event::Char(c)),
                
                glutin::WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
                    Some(glutin::VirtualKeyCode::Escape) => events.push(if input.state == glutin::ElementState::Pressed {
                        Event::KeyDown(Key::ToggleCursor)
                    } else {
                        Event::KeyUp(Key::ToggleCursor)
                    }),

                    _ => {}
                },

                glutin::WindowEvent::MouseWheel {
                    delta: glutin::MouseScrollDelta::LineDelta(_x, y), ..
                } => events.push(Event::Zoom(y as f32)),
                
                _ => {}
            },

            glutin::Event::DeviceEvent { event, .. } => match event {
                glutin::DeviceEvent::MouseMotion { delta: (dx, dy), .. } if cursor_grabbed =>
                    events.push(Event::CursorPan(Vec2::new(dx as f32, dy as f32))),

                _ => {}
            },

            _ => {}
        });

        events
    }

    pub fn swap_buffers(&self) -> Result<(), Error> {
        self.window
            .swap_buffers()
            .map_err(|err| Error::BackendError(Box::new(err)))
    }

    pub fn is_cursor_grabbed(&self) -> bool {
        self.cursor_grabbed;
    }

    pub fn grab_cursor(&mut self, grab: bool) {
        self.cursor_grabbed = grab;

        self.window.hide_cursor(grab);
        
        self.window.grab_cursor(grab)
            .expect("falha ao capturar ou deixar de capturar o cursor");
    }
}

/// representa uma chave que o jogo reconhece depois de um mapeamento de teclado
pub enum Key {
    ToggleCursor
}

/// representa um evento chegando da janela
pub enum Event {
    /// a janela que foi solicitada para ser fechada
    Close,

    /// a janela que será redimensionada
    Resize(Vec2<u32>),

    /// chave que foi digitada que corresponde ao caractere específico
    Char(char),

    /// cursor que foi paralizado ao redor da tela enquanto capturado
    CursorPan(Vec2<f32>),

    /// a câmera que foi solicitada para zoom
    Zoom(f32),

    /// chave que o jogo reconhece para ser pressionado para baixo
    keyDown(Key),

    /// chave que o jogo reconhece para ser lançado para baixo
    keyUp(Key)
}
