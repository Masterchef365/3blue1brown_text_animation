#version 450

layout(location = 0) in vec3 fragColor;

layout(location = 0) out vec4 outColor;

layout(set = 1, binding = 0) uniform Animation {
    float value;
} anim;

void main() {
    outColor = vec4(fragColor, 1.0);
}
