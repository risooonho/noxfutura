#version 450

layout(location=0) in vec2 v_tex_coords;
layout(location=0) out vec4 f_color;

layout(set = 0, binding = 0) uniform texture2D t_diffuse;
layout(set = 0, binding = 1) uniform sampler s_diffuse;
layout(set = 0, binding = 2) uniform texture2D t_normal;
layout(set = 0, binding = 3) uniform sampler s_normal;
layout(set = 0, binding = 4) uniform texture2D t_coords;
layout(set = 0, binding = 5) uniform sampler s_coords;

struct LightInfo {
    vec4 pos; // 4 contains the far_view
    vec4 color;
};

layout(set=1, binding=0) 
uniform Uniforms {
    vec4 screen_info;
    vec4 camera_position;
    LightInfo lights[32];
};

layout(set = 2, binding = 0) buffer LightMap {
    uint[] light_bits;
};

int mapidx(vec3 position) {
    int zc = int(round(position.y));
    int yc = int(round(position.z));
    int xc = int(round(position.x));
    return (zc * 256 * 256) + (yc * 256) + xc;
}

const vec3 samples[64]=vec3[64](
vec3(-0.047929014979204, -0.438413764902785, 0.929426961233746),
vec3(-0.269709204592704, 0.950515041224427, 0.206353989610903),
vec3(0.115462048337919, -0.973596029128394, -0.908342458086027),
vec3(0.0806885698758024, 0.925705718195549, -0.775657147370491),
vec3(-0.818854025110074, 0.552612993093465, -0.837888646794868),
vec3(0.917812085170002, 0.511669141026135, 0.482702973208565),
vec3(0.860502954135615, -0.643916628904123, -0.59160008969527),
vec3(0.4927422379035, 0.664258196344865, 0.351500938183611),
vec3(0.456449260085052, 0.451071517042271, 0.197256048385459),
vec3(-0.0770890598977241, -0.731329924012184, 0.256627452486744),
vec3(0.762439363558385, 0.289929235346325, -0.980721506556118),
vec3(0.805745881028283, 0.981082720275267, 0.959898172252732),
vec3(0.987773624658742, 0.894268979850048, 0.532481806436596),
vec3(0.991634111497466, 0.70068819104743, -0.763940778113785),
vec3(0.951780368759517, -0.0768451894573428, -0.500051180049417),
vec3(-0.240563757243764, 0.712237722010253, 0.174227118287188),
vec3(0.252475956057157, 0.853581860825418, 0.6421901210045),
vec3(-0.431675103916176, 0.641885146287785, 0.141548215387181),
vec3(-0.576182500075215, -0.516059089610147, -0.711348565943524),
vec3(0.716071723993237, 0.388852186242922, 0.812300858012269),
vec3(-0.935177913235506, 0.838241045743039, -0.838464823718813),
vec3(0.776158396358163, 0.0929938174762905, 0.292905676123544),
vec3(0.806935350679827, -0.838175281740873, 0.0393023892256006),
vec3(0.00745805126898125, -0.934154285074698, 0.605661123442571),
vec3(0.382660761354565, -0.77356647488387, -0.640886274395705),
vec3(0.61448898485048, 0.988673995403063, 0.116667312373815),
vec3(-0.909105247135674, -0.706164038343898, -0.481175743028916),
vec3(0.0462535540686926, 0.268954108655582, -0.902370086213705),
vec3(-0.228520638585718, 0.0409157975495393, -0.552664589745119),
vec3(0.571791567764772, 0.77897243466913, 0.469908848804465),
vec3(0.580044974780119, 0.837263924648454, 0.431036646984342),
vec3(-0.0489893534003558, 0.996032016268539, 0.768380880706172),
vec3(-0.558274395079814, 0.549639131300883, 0.27837004455577),
vec3(0.167467224381758, 0.0108904739165647, -0.790657199475527),
vec3(0.04403383418187, 0.895698401441043, -0.842476656419072),
vec3(0.938746585792893, -0.956280883354866, 0.837286858731445),
vec3(-0.443537157641154, -0.676352769266339, -0.128302908384674),
vec3(-0.87418601714471, 0.322312905774096, -0.759712795440762),
vec3(-0.724544917831573, -0.00221919553867789, -0.0967711298837752),
vec3(-0.975161909900947, 0.0565221001195153, 0.183158396566239),
vec3(0.818283460579766, -0.623341131409574, -0.731885250080112),
vec3(-0.637799055035528, -0.276806048820645, -0.426942824099501),
vec3(-0.516780444592215, 0.652402509594556, -0.0845697275223216),
vec3(0.455657960327755, -0.00613636484725588, 0.147927560100475),
vec3(0.636419369995088, -0.956138133925783, 0.0624363693983658),
vec3(0.276018297299378, 0.510623144362949, 0.716888975171009),
vec3(-0.826548283954511, -0.144072268338394, 0.596106303710207),
vec3(-0.794919454804488, 0.861427997013716, 0.131281364753063),
vec3(0.249229160214675, -0.507723697343464, 0.441255709612241),
vec3(0.0840510032785455, 0.401298733738271, 0.578262938348589),
vec3(-0.873061136588521, -0.742967173096409, 0.374191724415234),
vec3(-0.218813256326754, -0.270346983742737, -0.111649232343958),
vec3(-0.332886091609431, 0.569517305476091, 0.150298638205109),
vec3(-0.988458778680371, 0.887796161547977, 0.991071520382182),
vec3(-0.131937976781313, 0.26141419201547, 0.391276265635145),
vec3(0.976950409200295, 0.256124412582709, 0.844499972646254),
vec3(-0.716166136917728, -0.834360097309523, -0.986826448508014),
vec3(-0.764498859094076, -0.677069968946773, -0.991087930604148),
vec3(0.882030606279058, -0.390102148278822, 0.0877066348029119),
vec3(-0.339327945617814, -0.310451413094155, -0.563799889980217),
vec3(-0.990303448980524, -0.386055397295728, -0.461347742783093),
vec3(0.70775976348724, 0.772126136572357, 0.854851706342142),
vec3(-0.355966233058895, -0.146968676804214, 0.265689179962858),
vec3(-0.145761658046816, 0.373878060261686, 0.65622220492727)
);

layout(set = 3, binding = 0) buffer MouseBuffer {
    vec4[] mouse_buffer;
};

void main() {
    vec4 albedo = texture(sampler2D(t_diffuse, s_diffuse), v_tex_coords);
    vec3 normal = normalize(texture(sampler2D(t_normal, s_normal), v_tex_coords).rgb);
    vec4 raw_pos = texture(sampler2D(t_coords, s_coords), v_tex_coords);
    vec3 position = raw_pos.rgb;

    vec3 light_output = vec3(0.0, 0.0, 0.0);
    for (int i=0; i<32; ++i) {
        uint flag = 1 << i;

        int idx = mapidx(position);
        if ( (light_bits[idx] & flag) > 0 && (i==0 || int(lights[i].pos.y) == int(round(position.y - 0.4))) ) {

            float radius = lights[i].pos.a;
            float distance = distance(lights[i].pos.xyz, position);
            if (radius > 0.0 && distance < radius) {
                float attenuation = radius > 64.0 ? 1.0 : 1.0 / (distance*distance);
                vec3 lightDir = i > 0 ? normalize(lights[i].pos.rgb - position) : normalize(lights[i].pos.rgb);
                float diff = max(dot(normal, lightDir), 0.0);
                light_output += diff * albedo.rgb * lights[i].color.rgb * attenuation;

                // Phong
                vec3 viewDir = normalize(camera_position.xyz - position);
                vec3 reflectDir = reflect(-lightDir, normal);
                float spec = pow(max(dot(viewDir, reflectDir), 0.0), 32);
                vec3 specular = 0.5 * spec * lights[i].color.rgb * albedo.rgb;
                light_output += specular;
            }


        }
    }
    light_output += vec3(0.1) * albedo.rgb; // Ambient component

    // SSAO. Again.
    float SSAO = 0.0;
    const float kernelSize = 8.0;
    const float radius = 0.01;
    vec3 FragPos = vec3(v_tex_coords.xy, raw_pos.a);
    float occlusion = 0;
    for (int i=0; i<kernelSize; ++i) {
        vec2 mysample = FragPos.xy + (samples[i].xy * radius);
        float sampleDepth = texture(sampler2D(t_coords, s_coords), mysample).a;
        float rangeCheck = smoothstep(0.0, 1.0, radius / abs(FragPos.z - sampleDepth));
        occlusion += (sampleDepth >= FragPos.z ? 1.00 : 0.0) * rangeCheck;
    }
    SSAO = clamp((occlusion / kernelSize), 0.0, 1.0);

    // Write the color output
    f_color = vec4(light_output, 1.0) * vec4(SSAO, SSAO, SSAO, 1.0);

    // Write the mouse buffer output if we're at the mouse position
    if (int(gl_FragCoord.x) == int(screen_info.x) && int(gl_FragCoord.y) == int(screen_info.y)) {
        mouse_buffer[0] = vec4(position, 0.0);
    }
}