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

layout(set = 0, binding = 0) buffer Data {
    uint data[];
} inputData;

layout(constant_id = 0) const uint width = 1024;
layout(constant_id = 1) const uint height = 1024;

layout (push_constant) uniform Camera {
    mat4 matrix;
    uint drawGrid;
    uint flip;
    uvec2 position;
} camera;

layout(location = 0) out vec2 position;

void main() {
    if (camera.flip != 0) {
        uint index = camera.position.y * width + camera.position.x;
        inputData.data[index] = inputData.data[index] == 1 ? 0 : 1;
    }
    position = texcoords[gl_VertexIndex];
    gl_Position = camera.matrix * vec4(positions[gl_VertexIndex], 0.0, 1.0);
}