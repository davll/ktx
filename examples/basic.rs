extern crate futures_util;
extern crate gl;
extern crate glutin;
extern crate ktx_async as ktx;
extern crate lazy_static;
extern crate tokio;

use futures_util::stream::StreamExt as _;
use gl::types::*;
use lazy_static::lazy_static;
use std::path::Path;

#[tokio::main]
async fn main() {
    let event_loop = glutin::event_loop::EventLoop::new();
    let glctx = {
        let wb = glutin::window::WindowBuilder::new()
            .with_title("TestKTX: Basic")
            .with_inner_size(glutin::dpi::LogicalSize::new(1024.0, 768.0));
        let cb = glutin::ContextBuilder::new()
            .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGlEs, (2,0)));
        let ctx = cb.build_windowed(wb, &event_loop)
            .unwrap();
        unsafe { ctx.make_current().unwrap() }
    };

    gl::load_with(|s| glctx.get_proc_address(s) as *const _);

    let program = {
        let vscode = include_bytes!("simple.essl100.vert");
        let fscode = include_bytes!("simple.essl100.frag");
        let vs = compile_shader(gl::VERTEX_SHADER, vscode).unwrap();
        let fs = compile_shader(gl::FRAGMENT_SHADER, fscode).unwrap();
        let prog = link_program(&[vs, fs]).unwrap();
        unsafe {
            gl::DeleteShader(vs);
            gl::DeleteShader(fs);
        }
        prog
    };
    let position_attrib = unsafe {
        gl::GetAttribLocation(program, b"position\0".as_ptr() as *const GLchar)
    };
    let texcoord_attrib = unsafe {
        gl::GetAttribLocation(program, b"texcoord\0".as_ptr() as *const GLchar)
    };
    let tex_uniform = unsafe {
        gl::GetUniformLocation(program, b"tex\0".as_ptr() as *const GLchar)
    };

    let vertex_buffer = {
        let data = &[
            /* vx, vy, tx, ty */
            -0.5_f32, 0.5_f32, 0.0_f32, 0.0_f32,
            -0.5_f32, -0.5_f32, 0.0_f32, 1.0_f32,
            0.5_f32, 0.5_f32, 1.0_f32, 0.0_f32,
            0.5_f32, -0.5_f32, 1.0_f32, 1.0_f32,
        ];
        
        let mut obj: GLuint = 0;

        unsafe {
            gl::GenBuffers(1, &mut obj);
            gl::BindBuffer(gl::ARRAY_BUFFER, obj);
            gl::BufferData(gl::ARRAY_BUFFER,
                (data.len() * 4) as GLsizeiptr,
                data.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
        }

        obj
    };

    let texture = load_texture("data/pvr/block6.ktx").await;

    event_loop.run(move |event, _target, control_flow| {
        use glutin::event::{Event, WindowEvent};
        use glutin::event_loop::ControlFlow;

        //let next_frame_time = std::time::Instant::now() +
        //    std::time::Duration::from_nanos(16_666_667);
        //*control_flow = ControlFlow::WaitUntil(next_frame_time);
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::Resized(logical_size) => {
                    let dpi_factor = glctx.window().hidpi_factor();
                    glctx.resize(logical_size.to_physical(dpi_factor));
                },
                WindowEvent::RedrawRequested => {
                    // Clear Render Target
                    unsafe {
                        gl::Clear(gl::COLOR_BUFFER_BIT);
                    }

                    // Bind Program
                    unsafe {
                        gl::UseProgram(program);
                    }

                    // Update Uniform
                    unsafe {
                        if tex_uniform >= 0 {
                            gl::Uniform1i(tex_uniform, 0);
                        }
                    }

                    // Bind Texture
                    unsafe {
                        gl::ActiveTexture(gl::TEXTURE0);
                        gl::BindTexture(gl::TEXTURE_2D, texture);
                    }

                    // Bind Vertex Attrib
                    unsafe {
                        gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);
                        if position_attrib >= 0 {
                            let loc = position_attrib as GLuint;
                            let off = std::mem::transmute(0_usize);
                            gl::EnableVertexAttribArray(loc);
                            gl::VertexAttribPointer(loc, 2, gl::FLOAT, gl::FALSE, 16, off);
                        }
                        if texcoord_attrib >= 0 {
                            let loc = texcoord_attrib as GLuint;
                            let off = std::mem::transmute(8_usize);
                            gl::EnableVertexAttribArray(loc);
                            gl::VertexAttribPointer(loc, 2, gl::FLOAT, gl::FALSE, 16, off);
                        }
                    }

                    // Draw
                    unsafe {
                        gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
                    }

                    // Finalize and Present
                    unsafe {
                        gl::Flush();
                    }
                    glctx.swap_buffers().unwrap();
                },
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                    return;
                },
                _ => return,
            },
            Event::LoopDestroyed => {
                // Release OpenGL resources
                unsafe {
                    gl::DeleteBuffers(1, &vertex_buffer);
                    gl::DeleteTextures(1, &texture);
                    gl::DeleteProgram(program);
                }
                return;
            },
            _ => (),
        }
    });
}

fn compile_shader(ty: GLenum, src: &[u8]) -> Result<GLuint, String> {
    use std::ptr::null_mut;

    // Create Shader Object
    let id = unsafe { gl::CreateShader(ty) };

    // Import Shader Source
    unsafe {
        let len = src.len() as GLint;
        let ptr = src.as_ptr() as *const GLchar;
        gl::ShaderSource(id, 1, &ptr, &len);
    }

    // Compile
    unsafe {
        gl::CompileShader(id);
    }

    // Check Compilation Status
    let success = unsafe {
        let mut status: GLint = 0;
        gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut status);
        status == gl::TRUE as GLint
    };
    if success {
        Ok(id)
    } else {
        let len = unsafe {
            let mut x: GLint = 0;
            gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut x);
            (x) as usize
        };
        let mut buf: Vec<u8> = vec![0; len];
        unsafe {
            gl::GetShaderInfoLog(id, len as _, null_mut(), buf.as_mut_ptr() as _);
            gl::DeleteShader(id);
        }
        buf.pop(); // Remove trailing '\0'
        let e = String::from_utf8(buf).unwrap();
        Err(e)
    }
}

fn link_program(shader_objs: &[GLuint]) -> Result<GLuint, String> {
    use std::ptr::null_mut;

    // Create Program Object
    let id = unsafe { gl::CreateProgram() };

    // Attach Shaders
    for &shader_obj in shader_objs {
        unsafe {
            gl::AttachShader(id, shader_obj);
        }
    }

    // Link
    unsafe {
        gl::LinkProgram(id);
    }

    // Detach Shaders
    for &shader_obj in shader_objs {
        unsafe {
            gl::DetachShader(id, shader_obj);
        }
    }

    // Check Linkage Status
    let success = unsafe {
        let mut status: GLint = 0;
        gl::GetProgramiv(id, gl::LINK_STATUS, &mut status);
        status == gl::TRUE as GLint
    };
    if success {
        Ok(id)
    } else {
        let len = unsafe {
            let mut x: GLint = 0;
            gl::GetProgramiv(id, gl::INFO_LOG_LENGTH, &mut x);
            (x) as usize
        };
        let mut buf: Vec<u8> = vec![0; len];
        unsafe {
            gl::GetProgramInfoLog(id, len as _, null_mut(), buf.as_mut_ptr() as _);
            gl::DeleteProgram(id);
        }
        buf.pop(); // Remove trailing '\0'
        let e = String::from_utf8(buf).unwrap();
        Err(e)
    }
}

async fn load_texture(path: impl AsRef<Path>) -> GLuint {
    use tokio::fs::File;
    let file = File::open(PROJECT_DIR.join(path.as_ref())).await.unwrap();
    let decoder = ktx::Decoder::new(file);
    let (info, mut stream) = decoder.read_async().await.unwrap();

    let texture_obj = unsafe {
        let mut x: GLuint = 0;
        gl::GenTextures(1, &mut x);
        gl::BindTexture(gl::TEXTURE_2D, x);
        x
    };

    while let Some((frame, buf)) = stream.next().await.map(|r| r.unwrap()) {
        unsafe {
            gl::TexImage2D(gl::TEXTURE_2D,
                frame.level as GLint,
                info.gl_internal_format as GLint,
                frame.pixel_width as GLsizei,
                frame.pixel_height as GLsizei,
                /*border*/ 0,
                info.gl_format as GLenum,
                info.gl_type as GLenum,
                buf.as_ptr() as *const GLvoid);
        }
    }

    texture_obj
}

lazy_static! {
    static ref PROJECT_DIR: std::path::PathBuf = {
        use std::env::var_os;
        var_os("CARGO_MANIFEST_DIR")
            .map(|s| std::path::PathBuf::from(s)).unwrap()
    };
}
