#[derive(Copy, Clone)]
pub struct CubeVertex {
    pos: (f32, f32, f32),
}
implement_vertex!(CubeVertex, pos);

#[derive(Copy, Clone)]
pub struct Model {
    model: [[f32; 4]; 4],
}
implement_vertex!(Model, model);

pub const VERTICES: [CubeVertex; 8] = [
    CubeVertex {
        pos: (-1.0, 1.0, 1.0f32),
    },
    CubeVertex {
        pos: (-1.0, -1.0, 1.0),
    },
    CubeVertex {
        pos: (1.0, 1.0, 1.0),
    },
    CubeVertex {
        pos: (1.0, -1.0, 1.0),
    },
    CubeVertex {
        pos: (-1.0, 1.0, -1.0),
    },
    CubeVertex {
        pos: (-1.0, -1.0, -1.0),
    },
    CubeVertex {
        pos: (1.0, 1.0, -1.0),
    },
    CubeVertex {
        pos: (1.0, -1.0, -1.0),
    },
];

pub const INDICES: [u8; 3 * 12] = [
    0, 1, 2, 1, 3, 2, 0, 4, 5, 5, 1, 0, 2, 3, 6, 3, 7, 6, 1, 5, 3, 7, 3, 5, 0, 2, 4, 2, 6, 4, 7, 5,
    6, 4, 6, 5,
];

pub const VERT_SHADER: &'static str = r#"
#version 140

in vec3 pos;
in mat4 model;

uniform mat4 u_mvp;

out vec3 v_pos;


void main() {
    vec4 P = model*vec4(pos, 1.0);
    v_pos = P.xyz;
    gl_Position = u_mvp * P;
}
"#;

pub const FRAG_SHADER: &'static str = r#"
#version 140

in vec3 v_pos;

out vec4 colour;

void main() {
    colour = vec4(0.5 + 0.5*v_pos, 1.0);
}
"#;

/// Constructs model matrices for instanced viewing
///
/// Makes model matrices which can be used for instanced viewing of the cube
/// This creates multiple tightly packed model-matrices which constitutes a
/// dense cube, allowing the raycaster to enter the cube
pub fn make_matrices(n: usize) -> Vec<Model> {
    let mut data = Vec::with_capacity(n * n * n);

    let scale = 1.0 / n as f32;

    let dx = 2.0 / n as f32;

    for i in 0..n {
        let x = -1.0 + dx / 2.0 + i as f32 * dx;

        for j in 0..n {
            let y = -1.0 + dx / 2.0 + j as f32 * dx;

            for k in 0..n {
                let z = -1.0 + dx / 2.0 + k as f32 * dx;

                let translation_scaling = [
                    [scale, 0.0, 0.0, 0.0],
                    [0.0, scale, 0.0, 0.0],
                    [0.0, 0.0, scale, 0.0],
                    [x, y, z, 1.0],
                ];

                data.push(Model {
                    model: translation_scaling,
                })
            }
        }
    }
    data
}
