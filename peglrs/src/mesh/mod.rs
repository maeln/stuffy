use gl;
use std::os::raw::c_void;

// Note: This should probably be a trait.

#[derive(Debug)]
pub struct Mesh {
    pub vertices: Vec<f32>,
    pub indices: Option<Vec<u32>>,
    pub normals: Option<Vec<f32>>,
    pub uv: Option<Vec<f32>>,

    pub vbo_vertices: Option<u32>,
    pub vbo_indices: Option<u32>,
    pub vbo_normals: Option<u32>,
    pub vbo_uv: Option<u32>,
    pub vao: Option<u32>,

    pub v_components: i32,
    pub n_components: i32,
    pub uv_components: i32,
    pub draw_type: u32,
}

fn gen_vbo() -> Option<u32> {
    let mut vbo_addr: u32 = 0;
    unsafe {
        gl::GenBuffers(1, &mut vbo_addr);
    }
    Some(vbo_addr)
}

impl Drop for Mesh {
    fn drop(&mut self) {
        if self.vbo_vertices.is_some() {
            unsafe {
                gl::DeleteBuffers(1, &self.vbo_vertices.unwrap());
            }
        }

        if self.vbo_indices.is_some() {
            unsafe {
                gl::DeleteBuffers(1, &self.vbo_indices.unwrap());
            }
        }

        if self.vbo_normals.is_some() {
            unsafe {
                gl::DeleteBuffers(1, &self.vbo_normals.unwrap());
            }
        }

        if self.vbo_uv.is_some() {
            unsafe {
                gl::DeleteBuffers(1, &self.vbo_uv.unwrap());
            }
        }

        if self.vao.is_some() {
            unsafe {
                gl::DeleteVertexArrays(1, &self.vao.unwrap());
            }
        }
    }
}

impl Mesh {
    fn bind_vao(&mut self) {
        if self.vao.is_none() {
            let mut vao_addr: u32 = 0;
            unsafe {
                gl::GenVertexArrays(1, &mut vao_addr);
            }
            self.vao = Some(vao_addr);
        }
        unsafe {
            gl::BindVertexArray(self.vao.unwrap());
        }
    }

    fn free_vao(&self) {
        unsafe {
            gl::BindVertexArray(0);
        }
    }

    fn enable_attrib(&mut self) {
        self.bind_vao();

        if self.vbo_indices.is_some() {
            unsafe {
                gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo_vertices.unwrap());
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.vbo_indices.unwrap());
            }
        } else {
            unsafe {
                gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo_vertices.unwrap());
            }
        }

        unsafe {
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                self.v_components,
                gl::FLOAT,
                gl::FALSE,
                0,
                std::ptr::null_mut(),
            );
        }

        if self.vbo_normals.is_some() {
            unsafe {
                gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo_normals.unwrap());
                gl::EnableVertexAttribArray(1);
                gl::VertexAttribPointer(
                    1,
                    self.n_components,
                    gl::FLOAT,
                    gl::FALSE,
                    0,
                    std::ptr::null_mut(),
                );
            }
        }

        if self.vbo_uv.is_some() {
            unsafe {
                gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo_uv.unwrap());
                gl::EnableVertexAttribArray(2);
                gl::VertexAttribPointer(
                    2,
                    self.uv_components,
                    gl::FLOAT,
                    gl::FALSE,
                    0,
                    std::ptr::null_mut(),
                );
            }
        }

        self.free_vao();

        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }
    }

    fn upload(&mut self) {
        if self.vbo_vertices.is_none() {
            self.vbo_vertices = gen_vbo();
            unsafe {
                gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo_vertices.unwrap());
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (std::mem::size_of::<f32>() * self.vertices.len()) as isize,
                    self.vertices.as_mut_ptr() as *const c_void,
                    gl::STATIC_DRAW,
                );
            }
        }

        if self.vbo_indices.is_none() {
            if let Some(ind) = &mut self.indices {
                self.vbo_indices = gen_vbo();
                unsafe {
                    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.vbo_indices.unwrap());
                    gl::BufferData(
                        gl::ELEMENT_ARRAY_BUFFER,
                        (std::mem::size_of::<u32>() * ind.len()) as isize,
                        ind.as_mut_ptr() as *const c_void,
                        gl::STATIC_DRAW,
                    );
                }
            }
        }

        if self.vbo_normals.is_none() {
            if let Some(norms) = &mut self.normals {
                self.vbo_normals = gen_vbo();
                unsafe {
                    gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo_normals.unwrap());
                    gl::BufferData(
                        gl::ARRAY_BUFFER,
                        (std::mem::size_of::<f32>() * norms.len()) as isize,
                        norms.as_mut_ptr() as *const c_void,
                        gl::STATIC_DRAW,
                    );
                }
            }
        }

        if self.vbo_uv.is_none() {
            if let Some(uv) = &mut self.uv {
                self.vbo_uv = gen_vbo();
                unsafe {
                    gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo_uv.unwrap());
                    gl::BufferData(
                        gl::ARRAY_BUFFER,
                        (std::mem::size_of::<f32>() * uv.len()) as isize,
                        uv.as_mut_ptr() as *const c_void,
                        gl::STATIC_DRAW,
                    );
                }
            }
        }
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }
    }

    pub fn ready_up(&mut self) {
        self.upload();
        self.enable_attrib();
    }

    pub fn draw(&mut self) {
        self.bind_vao();

        if self.vbo_indices.is_some() {
            let fnb = self.indices.as_mut().map_or(0, |ind| ind.len() as i32);
            unsafe {
                gl::DrawElements(self.draw_type, fnb, gl::UNSIGNED_INT, std::ptr::null_mut());
            }
        } else {
            unsafe { gl::DrawArrays(self.draw_type, 0, (self.vertices.len() / 3) as i32) }
        }

        self.free_vao();
    }

    pub fn cube() -> Mesh {
        Mesh {
            v_components: 3,
            vertices: vec![
                -1.000000, -1.000000, 1.000000, -1.000000, 1.000000, 1.000000, -1.000000,
                -1.000000, -1.000000, -1.000000, 1.000000, -1.000000, 1.000000, -1.000000,
                1.000000, 1.000000, 1.000000, 1.000000, 1.000000, -1.000000, -1.000000, 1.000000,
                1.000000, -1.000000,
            ],
            indices: Some(vec![
                1, 2, 0, 3, 6, 2, 7, 4, 6, 5, 0, 4, 6, 0, 2, 3, 5, 7, 1, 3, 2, 3, 7, 6, 7, 5, 4, 5,
                1, 0, 6, 4, 0, 3, 1, 5,
            ]),
            n_components: 3,
            normals: Some(vec![
                -1.0000, 0.0000, 0.0000, 0.0000, 0.0000, -1.0000, 1.0000, 0.0000, 0.0000, 0.0000,
                0.0000, 1.0000, 0.0000, -1.0000, 0.0000, 0.0000, 1.0000, 0.0000,
            ]),
            uv_components: 2,
            uv: Some(vec![
                0.000200, 0.666866, 0.333134, 0.999800, 0.000200, 0.999800, 0.666866, 0.000200,
                0.999800, 0.333134, 0.666866, 0.333134, 0.333134, 0.666467, 0.000200, 0.333533,
                0.333134, 0.333533, 0.666467, 0.666467, 0.333533, 0.333533, 0.666467, 0.333533,
                0.333533, 0.333134, 0.666467, 0.000200, 0.666467, 0.333134, 0.000200, 0.333134,
                0.333134, 0.000200, 0.333134, 0.333134, 0.333134, 0.666866, 0.999800, 0.000200,
                0.000200, 0.666467, 0.333533, 0.666467, 0.333533, 0.000200, 0.000200, 0.000200,
            ]),
            vbo_vertices: None,
            vbo_indices: None,
            vbo_normals: None,
            vbo_uv: None,
            vao: None,
            draw_type: gl::TRIANGLES,
        }
    }

    pub fn fs_quad() -> Mesh {
        Mesh {
            v_components: 2,
            vertices: vec![
                -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 1.0,
            ],
            uv_components: 2,
            uv: Some(vec![
                0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0,
            ]),
            n_components: 0,
            normals: None,
            indices: None,
            vbo_vertices: None,
            vbo_indices: None,
            vbo_normals: None,
            vbo_uv: None,
            vao: None,
            draw_type: gl::TRIANGLES,
        }
    }
}
