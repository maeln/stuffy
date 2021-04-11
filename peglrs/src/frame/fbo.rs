#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ColorAttachment {
    RGBA_8B,
    RGBA_16F,
    RGBA_32F,
}

#[derive(Debug)]
pub enum DepthStencilAttachment {
    DEPTH24_STENCIL8,
}

#[derive(Debug)]
pub struct Framebuffer {
    pub addr: u32,
    pub color_attachment: Option<u32>,
    pub color_type: Option<ColorAttachment>,
    pub depth_stencil_attachment: Option<u32>,
    pub depth_stencil_type: Option<DepthStencilAttachment>,
}

pub fn make_color_attachment(attachment_type: ColorAttachment, width: i32, height: i32) -> u32 {
    let mut addr = 0;
    unsafe {
        gl::GenTextures(1, &mut addr);
        gl::BindTexture(gl::TEXTURE_2D, addr);

        match attachment_type {
            ColorAttachment::RGBA_8B => {
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGBA8 as i32,
                    width,
                    height,
                    0,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    std::ptr::null(),
                );
            }
            ColorAttachment::RGBA_16F => {
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGBA16F as i32,
                    width,
                    height,
                    0,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    std::ptr::null(),
                );
            }
            ColorAttachment::RGBA_32F => {
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGBA32F as i32,
                    width,
                    height,
                    0,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    std::ptr::null(),
                );
            }
        }

        gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_WRAP_S,
            gl::CLAMP_TO_BORDER as i32,
        );
        gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_WRAP_T,
            gl::CLAMP_TO_BORDER as i32,
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

        gl::BindTexture(gl::TEXTURE_2D, 0);
    }

    addr
}

pub fn make_depth_stencil_attachment(
    attachment_type: &DepthStencilAttachment,
    width: i32,
    height: i32,
) -> u32 {
    let mut addr = 0;

    unsafe {
        gl::GenRenderbuffers(1, &mut addr);
        gl::BindRenderbuffer(gl::RENDERBUFFER, addr);
        match attachment_type {
            &DepthStencilAttachment::DEPTH24_STENCIL8 => {
                gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, width, height);
            }
        }
        gl::BindRenderbuffer(gl::RENDERBUFFER, 0);
    }

    addr
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            if self.color_attachment.is_some() {
                gl::DeleteTextures(1, &self.color_attachment.unwrap());
            }
            if self.depth_stencil_attachment.is_some() {
                gl::DeleteRenderbuffers(1, &self.depth_stencil_attachment.unwrap());
            }
            gl::DeleteFramebuffers(1, &self.addr);
        }
    }
}

impl Framebuffer {
    pub fn new(
        color_attachment: ColorAttachment,
        depth_stencil_attachment: DepthStencilAttachment,
        width: i32,
        height: i32,
    ) -> Framebuffer {
        let color = make_color_attachment(color_attachment, width, height);
        let ds = make_depth_stencil_attachment(&depth_stencil_attachment, width, height);

        let mut addr = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut addr);
            gl::BindFramebuffer(gl::FRAMEBUFFER, addr);
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                color,
                0,
            );
            gl::FramebufferRenderbuffer(
                gl::FRAMEBUFFER,
                gl::DEPTH_STENCIL_ATTACHMENT,
                gl::RENDERBUFFER,
                ds,
            );
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        Framebuffer {
            addr,
            color_attachment: Some(color),
            color_type: Some(color_attachment),
            depth_stencil_attachment: Some(ds),
            depth_stencil_type: Some(depth_stencil_attachment),
        }
    }

    pub fn new_ldr(width: i32, height: i32) -> Framebuffer {
        Framebuffer::new(
            ColorAttachment::RGBA_8B,
            DepthStencilAttachment::DEPTH24_STENCIL8,
            width,
            height,
        )
    }

    pub fn new_hdr(width: i32, height: i32) -> Framebuffer {
        Framebuffer::new(
            ColorAttachment::RGBA_16F,
            DepthStencilAttachment::DEPTH24_STENCIL8,
            width,
            height,
        )
    }

    pub fn new_xhdr(width: i32, height: i32) -> Framebuffer {
        Framebuffer::new(
            ColorAttachment::RGBA_32F,
            DepthStencilAttachment::DEPTH24_STENCIL8,
            width,
            height,
        )
    }
}
