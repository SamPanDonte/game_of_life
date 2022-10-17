#version 460 core

layout(location = 0) in vec2 position;

layout(set = 0, binding = 0) readonly buffer Data {
    uint data[];
} inputData;

layout(constant_id = 0) const uint width = 1024;
layout(constant_id = 1) const uint height = 1024;

layout (push_constant) uniform Camera {
    mat4 matrix;
    uint drawGrid;
} camera;

layout(location = 0) out vec4 color;

void main() {
    vec2 positionScaled = position * vec2(uvec2(width, height));
    uvec2 index = uvec2(positionScaled);
    float value = inputData.data[index.x + index.y * width] == 1 ? 0.0 : 1.0;
    if (camera.drawGrid == 1 && value == 1.0 && (fract(positionScaled.x) < 0.1 || fract(positionScaled.y) < 0.1 || fract(positionScaled.x) > 0.9 || fract(positionScaled.y) > 0.9)) {
        value = 0.8;
    }
    color = vec4(value);
}