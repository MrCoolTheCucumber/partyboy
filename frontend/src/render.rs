use crate::{HEIGHT, SCALE, WIDTH};
use gameboy::GameBoy;
use gl::types::GLuint;
use std::ffi::c_void;

pub fn init_gl_state(tex_id: &mut u32, fb_id: &mut u32) {
    unsafe {
        gl::GenTextures(1, tex_id);
        gl::BindTexture(gl::TEXTURE_2D, *tex_id);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

        let mut data: [u8; (WIDTH * HEIGHT * 3) as usize] = [0; (WIDTH * HEIGHT * 3) as usize];
        let mut i = 0usize;
        while i < (WIDTH * HEIGHT * 3) as usize {
            data[i] = 55;
            data[i + 1] = 55;
            data[i + 2] = 55;

            i += 3;
        }

        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB as i32,
            WIDTH as i32,
            HEIGHT as i32,
            0,
            gl::RGB,
            gl::UNSIGNED_BYTE,
            data.as_ptr() as *const c_void,
        );

        gl::BindTexture(gl::TEXTURE_2D, 0);

        // https://stackoverflow.com/questions/31482816/opengl-is-there-an-easier-way-to-fill-window-with-a-texture-instead-using-vbo

        gl::GenFramebuffers(1, fb_id);

        gl::ClearColor(0.4549, 0.92549, 0.968627, 0.7);
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }
}

pub fn render_gb(gb: &GameBoy, fb_id: GLuint, tex_id: GLuint) {
    let frame_buffer = gb.get_frame_buffer();
    let mut tex_data = [0u8; (WIDTH * HEIGHT * 3) as usize];
    let mut i: usize = 0;

    while i < (WIDTH * HEIGHT * 3) as usize {
        let index = i / 3;
        let color = frame_buffer[index];

        tex_data[i] = color;
        tex_data[i + 1] = color;
        tex_data[i + 2] = color;

        i += 3;
    }

    unsafe {
        gl::BindTexture(gl::TEXTURE_2D, tex_id);
        gl::TexSubImage2D(
            gl::TEXTURE_2D,
            0,
            0,
            0,
            WIDTH as i32,
            HEIGHT as i32,
            gl::RGB,
            gl::UNSIGNED_BYTE,
            tex_data.as_ptr() as *const c_void,
        );
        gl::BindTexture(gl::TEXTURE_2D, 0);

        gl::BindFramebuffer(gl::READ_FRAMEBUFFER, fb_id);
        gl::FramebufferTexture2D(
            gl::READ_FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D,
            tex_id,
            0,
        );

        gl::BlitFramebuffer(
            0,
            0,
            WIDTH as i32,
            HEIGHT as i32,
            0,
            (HEIGHT * SCALE) as i32,
            (WIDTH * SCALE) as i32,
            0,
            gl::COLOR_BUFFER_BIT,
            gl::NEAREST,
        );
        gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
    }
}
