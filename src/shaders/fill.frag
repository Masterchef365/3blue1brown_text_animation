#version 450

layout(location = 0) in vec3 fragVars;

layout(location = 0) out vec4 outColor;

layout(set = 1, binding = 0) uniform Animation {
    float value;
} anim;

void main() {
    float x = (fragVars.x + 1.) / 2.;
    vec3 color = vec3(anim.value / 500.0 - x);//vec3((anim.value / 500.0) * x);
    outColor = vec4(color, 1.0);
}
