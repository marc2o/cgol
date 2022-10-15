/*
                                      ___
         ______    ___    ___   ___  /    \ ___
      _/       \_/    \_/ _  \_/   \_--   /-   \
     /   /  /  /   /  /   /__/  /__/   __/   / /
    /___/__/__/\__/\_/___/   \____/      \____/
    (c) 2022 Marc Oliver Orth     \______/
    https://marc2o.github.io

    An Implementation of Conway’s Game of Life
    (see https://en.wikipedia.org/wiki/Conway's_Game_of_Life)
    in Rust and SDL2.
*/

use std::thread::sleep;
use std::time::{
    Duration,
    Instant
};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::{
    Cursor,
    SystemCursor
};
use sdl2::rect::{
    Point,
    Rect
};
use sdl2::pixels::Color;
use sdl2::render::{
    Canvas,
    WindowCanvas,
    Texture,
    TextureCreator
};
use sdl2::video::{
    Window,
    WindowContext
};
use sdl2::{
    Sdl,
    EventPump
};

pub const TITLE: &str = "Conway’s Game Of Life";
pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
pub const SCREEN_OUTPUT: (u32, u32) = (800, 480); // 960, 540
pub const SCREEN_SOURCE: (u32, u32) = (400, 240);
pub const SCREEN_REFRESH_RATE: u32 = 1_000_000_000 / 60;
pub const GOL_CELL_W: u32 = 8;
pub const GOL_CELL_H: u32 = 8;
pub const GOL_COLS: i32 = 50;
pub const GOL_ROWS: i32 = 30;
pub const GOL_MEMORY_SIZE: usize = (GOL_COLS * GOL_ROWS) as usize;
pub const GOL_NEIGHBORS: [i32; 8] = [-GOL_COLS, GOL_COLS, -1, 1, -GOL_COLS - 1, -GOL_COLS + 1, GOL_COLS - 1, GOL_COLS + 1];

fn main() {
    // SDL2 STUFF

    let sdl_context: Sdl = match sdl2::init() {
        Ok(context) => context,
        Err(err) => panic!("Unable to initialize SDL2: {}", err),
    };

    let sdl_video = match sdl_context.video() {
        Ok(sdl_video) => sdl_video,
        Err(err) => panic!("Unable to access SDL2 video subsystem: {}", err)
    };

    let sdl_window = match sdl_video
        .window(TITLE, SCREEN_OUTPUT.0, SCREEN_OUTPUT.1)
        .position_centered()
        .opengl()
        .build() {
            Ok(sdl_window) => sdl_window,
            Err(err) => panic!("Unable to create window: {}", err)
        };

    let mut canvas: Canvas<Window> = match sdl_window
        .into_canvas()
        .index(find_sdl_gl_driver().unwrap())
        .build() {
            Ok(canvas) => canvas,
            Err(err) => panic!("Unable to create renderer from window: {}", err)
        };

    let creator: TextureCreator<WindowContext> = canvas.texture_creator();

    let mut buffer: Texture = creator
        .create_texture_target(
            sdl2::pixels::PixelFormatEnum::RGBA8888,
            SCREEN_SOURCE.0,
            SCREEN_SOURCE.1
            )
        .expect("Unable to create buffer.");

    let mut game_of_life = GameOfLife::new();
        
    // lightweight space ship (LWSS)
    game_of_life.memory[640] = 1;
    game_of_life.memory[641] = 1;
    game_of_life.memory[642] = 1;
    game_of_life.memory[643] = 1;
    game_of_life.memory[689] = 1;
    game_of_life.memory[693] = 1;
    game_of_life.memory[743] = 1;
    game_of_life.memory[792] = 1;
    
    // Blinker
    game_of_life.memory[310] = 1;
    game_of_life.memory[360] = 1;
    game_of_life.memory[410] = 1;
    
    // Glider
    game_of_life.memory[1040] = 1;
    game_of_life.memory[1088] = 1;
    game_of_life.memory[1090] = 1;
    game_of_life.memory[1139] = 1;
    game_of_life.memory[1140] = 1;
    
    // MAIN LOOP
    let mut event_pump = sdl_context.event_pump().unwrap();
    
    let mut is_running: bool = true;
    
    while is_running {
        let t0 = Instant::now();
        
        is_running = game_of_life.handle_events(&mut event_pump);

        game_of_life.update();
        
        let _result = canvas
            .with_texture_canvas(&mut buffer, |texture_canvas| {
                game_of_life.draw(texture_canvas);
            });

        render_frame(&buffer, &mut canvas);
        
        let dt: u32 = t0.elapsed().as_nanos().try_into().unwrap();
        if dt < SCREEN_REFRESH_RATE {
            sleep(Duration::new(0, SCREEN_REFRESH_RATE - dt));
        }
    }
}

// -------------------------------------
// --- UTILITY FUNCTIONS
// -------------------------------------

fn find_sdl_gl_driver() -> Option<u32> {
    for (index, item) in sdl2::render::drivers().enumerate() {
        if item.name == "opengl" {
            return Some(index as u32);
        }
    }

    None
}

fn render_frame(buffer: &Texture, canvas: &mut WindowCanvas) {
    canvas.copy(buffer, None, None).unwrap();
    canvas.present();
}

pub fn wrap(mut n: i32, min: i32, max: i32) -> i32 {
    while n < min {
        n += max - min;
    }
    while n >= max {
        n -= max - min;
    }

    return n;
}

pub fn clamp(low: f32, val: f32, high: f32) -> f32 {
    val.max(low).min(high)
}


// -------------------------------------
// --- STRUCTURES & IMPLEMENTATIONS
// -------------------------------------

pub struct ColorPalette {
    pub palette: Vec<(u8, u8, u8)>
}
impl ColorPalette {
    pub fn new() -> Self {
        let palette = Vec::new();
        ColorPalette {
            palette
        }
    }

    pub fn set(palette: Vec<(u8, u8, u8)>) -> Self {
        ColorPalette {
            palette
        }
    }

    pub fn get_color(&self, n: usize) -> Color {
        Color::RGB(
            self.palette[n].0,
            self.palette[n].1,
            self.palette[n].2
            )
    }
}

#[derive(PartialEq)]
pub enum GameMode {
    Edit,
    Play,
    Stop
}

pub struct GameOfLife {
    pub mode: GameMode,
    pub dark: bool,
    pub colors: ColorPalette,
    pub frames: (i32, i32),
    pub memory: [i32; GOL_MEMORY_SIZE],
    pub edit_cursor: Cursor,
    pub play_cursor: Cursor
}
impl GameOfLife {
    pub fn new() -> Self {
        let colors: ColorPalette = ColorPalette::set(vec![
            // https://lospec.com/palette-list/2bit-demichrome
            (0x22, 0x22, 0x23), // blackish
            (0xF0, 0xF6, 0xF0), // whitish
        ]);
        
        let edit_cursor: Cursor = Cursor::from_system(SystemCursor::Crosshair).unwrap();
        let play_cursor: Cursor = Cursor::from_system(SystemCursor::Arrow).unwrap();

        GameOfLife {
            mode: GameMode::Play,
            dark: true,
            colors,
            frames: (16, 16),
            memory: [0; GOL_MEMORY_SIZE],
            edit_cursor,
            play_cursor
        }
    }

    pub fn set_mode(&mut self, mode: GameMode) {
        self.mode = mode;        
    }

    pub fn update(&mut self) {
        match self.mode {
            GameMode::Edit => {}
            GameMode::Play => {
                self.frames.1 -= 1;
                if self.frames.1 <= 0 {
                    self.frames.1 = self.frames.0;

                    self.animate();
                }
            }
            GameMode::Stop => {}
        }
    }

    fn animate(&mut self) {
        let mut copy_of_cells: [i32; GOL_MEMORY_SIZE] = [0; GOL_MEMORY_SIZE];
    
        for i in 1 .. self.memory.len() {
            copy_of_cells[i] = self.memory[i];
        }
    
        for i in 1 .. copy_of_cells.len() {
            let c: i32 = copy_of_cells[i];
            let mut n: i32 = 0;
        
            for neighbor in GOL_NEIGHBORS.iter() {
                if copy_of_cells[wrap(neighbor + i as i32, 0, GOL_MEMORY_SIZE as i32) as usize] == 1 {
                    n += 1;
                }
            }
        
            // The Rules:
            // 1. Any live cell with two or three live neighbors
            // survives.
            if (n >= 2 && n <= 3) && c == 1 {
                self.memory[i] = 1;
            }
            // 2. Any dead cell with three live neighbors
            // becomes a live cell.
            else if n == 3 && c == 0 {
                self.memory[i] = 1;
            }
            // 3. All other live cells die in the next generation.
            // Similarly, all other dead cells stay dead.
            else {
                self.memory[i] = 0;
            }
        }
    
    }
    
    pub fn draw(&mut self, target: &mut WindowCanvas) {
        let background: usize;
        let foreground: usize;
        if self.dark {
            background = 0;
            foreground = 1;
        } else {
            background = 1;
            foreground = 0;
        }
        target.set_draw_color(self.colors.get_color(background));
        target.clear();    

        target.set_draw_color(self.colors.get_color(foreground));

        match self.mode {
            GameMode::Edit => {
                // draw pixel grid in edit mode
                let mut points: [Point; 1450] = [Point::new(0, 0); 1450];
                for i in 0 .. points.len() {
                    points[i] = Point::new(
                        i as i32 % 50 * 8 + 8,
                        i as i32 / 50 * 8 + 8
                    );
                }
                let _result = target.draw_points(&points[..]);
            }
            GameMode::Play => {}
            GameMode::Stop => {}
        }
   
        // draw memory aka the cells
        for i in 1 .. self.memory.len() {
            if self.memory[i] == 1 {
                let _result = target.fill_rect(Rect::new(
                    i as i32 % 50 * 8,
                    i as i32 / 50 * 8,
                    GOL_CELL_W,
                    GOL_CELL_H
                ));
            }
        }
    }

    // Esc      quit program
    // Return   toggle edit / play mode
    // Left MB  draw cells in edit mode
    // Right MB clear cells in edit mode
    // F1       dark / light mode
    pub fn handle_events(&mut self, events: &mut EventPump) -> bool {
        let mut is_running = true;
        
        if self.mode == GameMode::Edit &&
            (events.mouse_state().left() || events.mouse_state().right()) {            
            let col = (events.mouse_state().x() / 2 / 8) % 50;
            let row = (events.mouse_state().y() / 2 / 8) % 30;        
            let idx = (row * 50 + col) as usize;

            if events.mouse_state().left() {
                self.memory[idx] = 1;
            } else if events.mouse_state().right() {
                self.memory[idx] = 0;
            }
        }
        
        for event in events.poll_iter() {
            match event {
                Event::MouseButtonDown { mouse_btn: _, x: _, y: _, ..} => {
                    /* if mouse_btn == MouseButton::Left 
                    && self.mode == GameMode::Edit {
                        let idx = (m_row * 50 + m_col) as usize;
                        
                        if self.memory[idx] == 1 {
                            self.memory[idx] = 0;
                        } else {
                            self.memory[idx] = 1;
                        }
                    }*/
                }
                
                Event::KeyUp { keycode: Some(Keycode::Return), .. } => {
                    match self.mode {
                        GameMode::Play => {
                            self.set_mode(GameMode::Edit);
                            self.edit_cursor.set();
                        }
                        GameMode::Edit => {
                            self.set_mode(GameMode::Play);
                            self.play_cursor.set();
                        }
                        
                        _ => {}
                    }
                    
                }
                Event::KeyUp { keycode: Some(Keycode::F1), .. } => {
                    self.dark = !self.dark;
                }
                
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    is_running = false;
                }

                _ => (),
            }
        }

        is_running
    }
}
