#version 450

layout(location=0) in vec3 v_tint;
layout(location=1) in vec3 v_world_pos;
layout(location=2) in vec2 v_uv;
layout(location=3) flat in int v_material;
layout(location=4) flat in mat3 v_tbn;
layout(location=10) in vec3 v_normal;

layout(location=0) out vec4 f_color;
layout(location=1) out vec4 f_normal;
layout(location=2) out vec4 f_pbr;
layout(location=3) out vec4 f_coords;


layout(set = 1, binding = 0) uniform texture2D t_terrain;
layout(set = 1, binding = 1) uniform sampler s_terrain;

vec4 sample_material(int tex_index, vec2 uv) {
    int tex_x = tex_index % 16;
    int tex_y = tex_index / 16;

    vec2 tex_base = vec2(
        float(tex_x),
        float(tex_y)
    ) / 16.0;

    vec2 terrain_uv = vec2(
        (uv.x + tex_base.x),
        (uv.y + tex_base.y)
    );
    return texture(sampler2D(t_terrain, s_terrain), tex_base + uv);
    //return vec4(tex_base + uv, 0.0, 1.0);
}

vec4 sample_material_exact(int tex_index, vec2 uv) {
    int tex_x = tex_index % 16;
    int tex_y = tex_index / 16;

    vec2 tex_base = vec2(
        float(tex_x),
        float(tex_y)
    ) / 16.0;

    vec2 terrain_uv = vec2(
        (uv.x + tex_base.x),
        (uv.y + tex_base.y)
    );
    return textureLod(sampler2D(t_terrain, s_terrain), tex_base + uv, 0.0);
    //return vec4(tex_base + uv, 0.0, 1.0);
}

float LinearizeDepth()
{
  float z = gl_FragCoord.z;
  return ((z + 1.0) / 2.0) / 256.0;
}

void main() {
    int mat_base = int(v_material);
    vec2 uv = vec2(
        clamp(fract(v_uv.x), 0.01, 0.99) / 16.0,
        clamp(fract(v_uv.y), 0.01, 0.99) / 16.0
    );

    vec4 terrain_color = sample_material(mat_base, uv);
    vec3 tex_normal = sample_material(mat_base + 1, uv).rgb;
    tex_normal = normalize(tex_normal * 2.0 - 1.0);
    vec3 normal = normalize(v_tbn * tex_normal);
    vec3 pbr = sample_material(mat_base + 2, uv).rgb;

    f_color = terrain_color * vec4(v_tint, 1.0);
    f_normal = vec4(normal, 1.0);
    f_pbr = vec4(
        pbr.r, // AO
        pbr.g, // Rough
        pbr.b, // Metal
        1.0
    );
    f_coords = vec4(v_world_pos, gl_FragCoord.z);
}