use glium::implement_vertex;

#[derive(Copy, Clone)]
pub struct QuadVertex {
    pos: (f32, f32),
}

implement_vertex!(QuadVertex, pos);

pub const VERTICES: [QuadVertex; 4] = [
    QuadVertex { pos: (-1.0, -1.0) },
    QuadVertex { pos: (-1.0, 1.0) },
    QuadVertex { pos: (1.0, -1.0) },
    QuadVertex { pos: (1.0, 1.0) },
];

pub const INDICES: [u8; 6] = [0, 1, 2, 1, 3, 2];

pub const VERT_SHADER: &'static str = r#"
    #version 140

    in vec2 pos;

    out vec2 v_pos;

    void main() {
        v_pos = 0.5 + 0.5*pos;
        gl_Position = vec4(pos, 0.0, 1.0);
    }
    "#;

pub const FRAG_SHADER: &'static str = r#"
    #version 140

    in vec2 v_pos;

    uniform sampler2D u_back;
    uniform sampler2D u_front;
    uniform sampler3D u_volume;

    uniform sampler2D u_noise;
    uniform bool u_use_noise;

    uniform int u_steps;
    uniform float u_dx;
    uniform float u_gamma;



    uniform int u_mode; // 0 : MPI, 1 : ISO

    uniform vec3 u_colour;
    uniform float u_iso;


    uniform float u_dr;
    uniform vec3 u_L;

    uniform vec3 u_ambient;
    uniform float u_amb_str;

    uniform vec3 u_diffuse;
    uniform float u_dif_str;

    uniform vec3 u_specular;
    uniform float u_spe_str;
    uniform float u_alpha;

    out vec4 colour;

    vec4 gamma_correct(vec4 colour, float gamma_factor) {
        return vec4(pow(colour.rgb, vec3(1.0/gamma_factor)), colour.a);
    }

    void main() {
        if (texture(u_front, v_pos).a == 0) {
            colour = vec4(0.0);
            return;
        }

        vec3 start = texture(u_front, v_pos).xyz;
        vec3 end   = texture(u_back,  v_pos).xyz;


        int n = int (floor(distance(end, start) / u_dx)) - 2;

        vec3 direction = normalize(end - start);

        vec3 ray = start;
        if (u_use_noise) {
            ray += u_dx*direction*texture(u_noise, v_pos).r;
        }

        int max_iterations = min(u_steps, n);

        if (u_mode == 0) { // Maximum Intensity Projection
            float max_found = -1.0;

            for (int i = 0; i < max_iterations; i++) {
                ray += direction*u_dx;
                float potential = texture(u_volume, ray).r;
                max_found = max(max_found, potential);
            }

            if (max_found > 0.0) {
                colour = gamma_correct(vec4(max_found*u_colour, 1.0), u_gamma);
            } else {
                colour = vec4(0.0);
            }

            return;
        } else if (u_mode == 1) { // Isosurface extraction

            for (int i = 0; i < max_iterations; i++) {
                ray += direction*u_dx;

                float potential = texture(u_volume, ray).r;

                if (potential > u_iso) {
                    vec3 gradient;
                    gradient.x = (texture(u_volume, vec3(ray.x + u_dr, ray.y, ray.z)).r - potential)/u_dr;
                    gradient.y = (texture(u_volume, vec3(ray.x, ray.y + u_dr, ray.z)).r - potential)/u_dr;
                    gradient.z = (texture(u_volume, vec3(ray.x, ray.y, ray.z + u_dr)).r - potential)/u_dr;
                    gradient = normalize(gradient);


                    vec3 ambient = u_amb_str*u_ambient;
                    vec3 diffuse = u_dif_str*u_diffuse*max(dot(gradient, u_L), 0.0);

                    vec3 H = normalize(u_L + direction);
                    vec3 specular = u_spe_str*u_specular*pow(dot(gradient, H), u_alpha) * (u_alpha + 8.0) / 8.0;

                    colour = gamma_correct(vec4(ambient + diffuse + specular, 1.0), u_gamma);
                    return;
                }
            }
        colour = vec4(0.0);
        }
    }
"#;
