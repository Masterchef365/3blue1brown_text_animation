#version 450

layout(location = 0) in vec2 inPosition;
layout(location = 1) in float inValue;

layout(location = 0) out vec3 fragVars;

layout(set = 0, binding = 0) uniform Camera {
    mat4 matrix;
} cam;

void main() {
    vec4 position = cam.matrix * vec4(inPosition, 0.0, 1.0);
    gl_Position = position;
    fragVars = vec3(position.xy, inValue);
}

