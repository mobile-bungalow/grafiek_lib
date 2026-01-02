#version 450

#pragma input(image, name=image)
layout(set = 0, binding = 0) uniform sampler default_sampler;
layout(set = 0, binding = 1) uniform texture2D image;

#pragma input(float, name="mix_amount", default=1.0, min=0.0, max=1.0)
layout(set = 0, binding = 2) uniform Inputs {
    float mix_amount;
};

layout(location = 0) out vec4 out_color;

void main() {
    ivec2 size = textureSize(sampler2D(image, default_sampler), 0);
    vec2 uv = gl_FragCoord.xy / vec2(size);

    vec4 color = texture(sampler2D(image, default_sampler), uv);

    float luminance = dot(color.rgb, vec3(0.2126, 0.7152, 0.0722));
    vec3 gray = vec3(luminance);

    out_color = vec4(mix(color.rgb, gray, mix_amount), color.a);
}
