use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::{Color, PixelFormat},
    rect, render,
};

pub mod constants;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window(
            constants::TITLE,
            constants::WIDTH as u32,
            constants::HEIGHT as u32,
        )
        .position_centered()
        .input_grabbed()
        .build()
        .unwrap();

    sdl_context.mouse().capture(true);
    sdl_context.mouse().set_relative_mouse_mode(true);

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut renderer = window
        .into_canvas()
        .present_vsync()
        .accelerated()
        .build()
        .unwrap();

    let mut player_dir = [0_f32; 2];
    let mut state: State = State {
        pos: [12., 12.],
        angle: 0.,
    };

    let fov = 60_f32.to_radians();

    'r: loop {
        renderer.set_draw_color(Color::RGB(0, 0, 0));
        renderer.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'r,
                Event::KeyDown { keycode, .. } => {
                    let k = keycode.unwrap();
                    match k {
                        Keycode::Escape => {
                            break 'r;
                        }
                        Keycode::Right | Keycode::D => {
                            state.rotate(2.5);
                        }
                        Keycode::Left | Keycode::A | Keycode::Q => {
                            state.rotate(-2.5);
                        }
                        Keycode::Up | Keycode::W | Keycode::Z => {
                            state.pos[0] += player_dir[0].round();
                            state.pos[1] += player_dir[1].round();
                        }
                        Keycode::Down | Keycode::S => {
                            let old_pos = [state.pos[0], state.pos[1]];
                            state.pos[0] -= player_dir[0].round();
                            state.pos[1] -= player_dir[1].round();
                            if out_of_bounds(&[state.pos[0] + 1., state.pos[1] + 1.]) {
                                state.pos = old_pos;
                            }
                        }
                        _ => {}
                    }
                }
                Event::MouseMotion { xrel, .. } => {
                    state.rotate(xrel as f32 / 2.);
                }
                _ => {}
            }
        }

        player_dir[0] = -1. * state.angle.to_radians().cos() - 0. * state.angle.to_radians().sin();
        player_dir[1] = -1. * state.angle.to_radians().sin() + 0. * state.angle.to_radians().cos();

        for x in 0..=constants::WIDTH {
            let result = ray(x as u32, &state.pos, &player_dir, fov);
            let mut rgb = 0x000000_u32;

            match result.hit_val {
                1 => {
                    rgb = 0xFFFF0000;
                }
                2 => {
                    rgb = 0xFF0000FF;
                }
                _ => {}
            }

            if result.side == 1 {
                // darken color
                // definitly didn't yank this code
                let rgb = 0xFF000000
                    | (((rgb & 0xFF00FF) * 0xC0) >> 8 & 0xFF00FF)
                    | (((rgb & 0x00FF00) * 0xC0) >> 8 & 0x00FF00);
                let color = Color::from_u32(
                    &PixelFormat::try_from(renderer.default_pixel_format()).unwrap(),
                    rgb,
                );
                renderer.set_draw_color(color);
            } else {
                let color = Color::from_u32(
                    &PixelFormat::try_from(renderer.default_pixel_format()).unwrap(),
                    rgb,
                );
                renderer.set_draw_color(color);
            }
            verline(
                &mut renderer,
                x,
                result.start_y as i32,
                result.length as i32,
            );
        }

        renderer.present();
    }
}

fn verline<T: render::RenderTarget>(renderer: &mut render::Canvas<T>, x: i32, y: i32, length: i32) {
    renderer
        .draw_line(rect::Point::new(x, y), rect::Point::new(x, y + length))
        .unwrap();
}

fn ray(x: u32, player_pos: &[f32; 2], player_dir: &[f32; 2], fov: f32) -> CastResult {
    let ray_angle = fov / constants::WIDTH as f32;
    let ray_dir = [
        player_dir[0] * (fov / 2. - x as f32 * ray_angle).cos()
            - player_dir[1] * (fov / 2. - x as f32 * ray_angle).sin(),
        player_dir[0] * (fov / 2. - x as f32 * ray_angle).sin()
            + player_dir[1] * (fov / 2. - x as f32 * ray_angle).cos(),
    ];

    let mut pos = [player_pos[0], player_pos[1]];
    let mut wall_dist_perp = 0.;
    let mut hit = false;
    let mut side = 0_i32;
    let mut hit_val = 0_u8;

    while !hit {
        let step_x;
        let step_y;

        let delta_dist_x = if ray_dir[0] == 0. {
            f32::INFINITY
        } else {
            (1. + (ray_dir[1] * ray_dir[1]) / (ray_dir[0] * ray_dir[0])).sqrt() / ray_dir[0].abs()
        };
        let delta_dist_y = if ray_dir[1] == 0. {
            f32::INFINITY
        } else {
            (1. + (ray_dir[0] * ray_dir[0]) / (ray_dir[1] * ray_dir[1])).sqrt() / ray_dir[1].abs()
        };

        let mut side_dist_x;
        let mut side_dist_y;

        if ray_dir[0] < 0. {
            side_dist_x = (player_pos[0] - pos[0]) * delta_dist_x;
            step_x = -1;
        } else {
            side_dist_x = (pos[0] + 1. - player_pos[0]) * delta_dist_x;
            step_x = 1;
        };
        if ray_dir[1] < 0. {
            side_dist_y = (player_pos[1] - pos[1]) * delta_dist_y;
            step_y = -1;
        } else {
            side_dist_y = (pos[1] + 1. - player_pos[1]) * delta_dist_y;
            step_y = 1;
        };

        if side_dist_x < side_dist_y {
            side_dist_x += delta_dist_x;
            pos[0] += step_x as f32;
            side = 0;
        } else {
            side_dist_y += delta_dist_y;
            pos[1] += step_y as f32;
            side = 1;
        }

        if !out_of_bounds(&pos) && {
            hit_val = constants::MAP_DATA[pos[1] as usize][pos[0] as usize];
            hit_val
        } > 0
        {
            hit = true;
            wall_dist_perp = if side == 0 {
                side_dist_x - delta_dist_x
            } else {
                side_dist_y - delta_dist_y
            };
        }
    }

    let length = (constants::HEIGHT as f32 / wall_dist_perp * ray_angle.cos()).round() as u32;
    let start_y = (constants::HEIGHT as i32 / 2 - length as i32 / 2).max(0) as u32;
    CastResult {
        start_y,
        length,
        side,
        hit_val,
    }
}

fn out_of_bounds(pos: &[f32; 2]) -> bool {
    pos[0] >= 0.
        && (pos[0] as u32 + 1) < constants::MAP_WIDTH - 1
        && pos[1] >= 0.
        && (pos[1] as u32 + 1) < constants::MAP_HEIGHT - 1
}

struct State {
    pos: [f32; 2],
    angle: f32,
}

struct CastResult {
    start_y: u32,
    length: u32,
    side: i32,
    hit_val: u8,
}

impl State {
    fn rotate(self: &mut State, amount: f32) {
        self.angle -= amount;
        self.angle = ((self.angle + 180.) % 360.) - 180.
    }
}
