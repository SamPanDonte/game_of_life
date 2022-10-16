#version 460 core

const vec2 positions[4] = {
    vec2(-1.0, -1.0),
    vec2(1.0, -1.0),
    vec2(-1.0, 1.0),
    vec2(1.0, 1.0),
};

const uvec2 texcoords[4] = {
    uvec2(0, 0),
    uvec2(1, 0),
    uvec2(0, 1),
    uvec2(1, 1),
};

layout (push_constant) uniform Camera {
    mat4 matrix;
} camera;

layout(location = 0) out vec2 position;

void main() {
    position = texcoords[gl_VertexIndex];
    gl_Position = camera.matrix * vec4(positions[gl_VertexIndex], 0.0, 1.0);
}