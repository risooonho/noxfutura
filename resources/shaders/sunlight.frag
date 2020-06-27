#version 450

layout(location=0) in vec2 v_tex_coords;
layout(location=0) out vec4 f_color;

layout(set = 1, binding = 0) uniform texture2D t_diffuse;
layout(set = 1, binding = 1) uniform sampler s_diffuse;
layout(set = 1, binding = 2) uniform texture2D t_normal;
layout(set = 1, binding = 3) uniform sampler s_normal;
layout(set = 1, binding = 4) uniform texture2D t_pbr;
layout(set = 1, binding = 5) uniform sampler s_pbr;
layout(set = 1, binding = 6) uniform texture2D t_coords;
layout(set = 1, binding = 7) uniform sampler s_coords;

// Uniform with the light position and transformation
layout(set=0, binding=0) 
uniform Uniforms {
    vec3 sun_pos;
    vec3 sun_color;
    vec3 camera_position;
};

#define PI 3.1415926

float DistributionGGX(vec3 N, vec3 H, float roughness)
{
    float a = roughness*roughness;
    float a2 = a*a;
    float NdotH = max(dot(N, H), 0.0);
    float NdotH2 = NdotH*NdotH;

    float nom   = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return nom / denom;
}

float GeometrySchlickGGX(float NdotV, float roughness)
{
    float r = (roughness + 1.0);
    float k = (r*r) / 8.0;

    float nom   = NdotV;
    float denom = NdotV * (1.0 - k) + k;

    return nom / denom;
}

float GeometrySmith(vec3 N, vec3 V, vec3 L, float roughness)
{
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx2 = GeometrySchlickGGX(NdotV, roughness);
    float ggx1 = GeometrySchlickGGX(NdotL, roughness);

    return ggx1 * ggx2;
}

vec3 fresnelSchlick(float cosTheta, vec3 F0)
{
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}

vec3 CalculateLightOutput(vec3 albedo, vec3 N, vec3 V, vec3 F0, vec3 L, vec3 radiance, float roughness, float metallic) {
    vec3 H = normalize(V + L);
    float NDF = DistributionGGX(N, H, roughness);
    float G   = GeometrySmith(N, V, L, roughness);
    vec3 F    = fresnelSchlick(max(dot(H, V), 0.0), F0);

    vec3 nominator    = NDF * G * F;
    float denominator = 4 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.001; // 0.001 to prevent divide by zero.
    vec3 specular = nominator / denominator;

    // kS is equal to Fresnel
    vec3 kS = F;
    // for energy conservation, the diffuse and specular light can't
    // be above 1.0 (unless the surface emits light); to preserve this
    // relationship the diffuse component (kD) should equal 1.0 - kS.
    vec3 kD = vec3(1.0) - kS;
    // multiply kD by the inverse metalness such that only non-metals
    // have diffuse lighting, or a linear blend if partly metal (pure metals
    // have no diffuse light).
    kD *= 1.0 - metallic;

    // scale light by NdotL
    float NdotL = max(dot(L, N), 0.0);

    return (kD * (albedo / PI) + specular) * NdotL;
}

vec3 GameLight(float far_plane, vec3 albedo, vec3 N, vec3 V, vec3 F0, float roughness, float metallic, vec3 light_position, vec3 position, vec3 light_color) {
    vec3 L = far_plane < 512.0 ? normalize(light_position - position) : normalize(light_position);
    float distance = distance(light_position.xyz, position);
    float attenuation = far_plane > 64.0 ? 1.0 : (1.0 / (distance));
    //float attenuation = 1.0;
    vec3 radiance = light_color * attenuation;

    // For simple light output calculation
    //float diff = max(dot(L, N), 0.0);
    //return albedo * diff;

    return CalculateLightOutput(albedo, N, V, F0, L, radiance, roughness, metallic) * radiance.r;
    //return light_color * attenuation;
}

void main() {
    float gamma = 2.2;
    vec3 albedo = texture(sampler2D(t_diffuse, s_diffuse), v_tex_coords).rgb;
    vec3 position = texture(sampler2D(t_coords, s_coords), v_tex_coords).rgb;
    vec3 normal = normalize(texture(sampler2D(t_normal, s_normal), v_tex_coords).rgb * 2.0 - 1.0);

    vec3 material_lookup = pow(texture(sampler2D(t_pbr, s_pbr), v_tex_coords).rgb, vec3(gamma));
    float ao = material_lookup.r;
    float rough = material_lookup.g;
    float metal = material_lookup.b;

    vec3 V = normalize(camera_position - position);
    vec3 F0 = mix(vec3(0.04), albedo, metal);

    vec3 light_output = vec3(0.0, 0.0, 0.0);
    light_output += GameLight(512.0, albedo, normal, V, F0, rough, metal, sun_pos, position, sun_color);
    light_output += vec3(0.03) * albedo * ao; // Ambient component

    // Map it - remove when layering
    //light_output = light_output / (light_output + vec3(1.0));
    //light_output = pow(light_output, vec3(1.0/2.2));

    f_color = vec4(light_output, 1.0);
}