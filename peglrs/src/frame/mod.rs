pub mod fbo;

use fbo::Framebuffer;

pub trait Frame {
    fn attach_fbo(fbo: Framebuffer);
    fn detach_fbo();

    fn draw();

    // Add scene + uniform managment
}
