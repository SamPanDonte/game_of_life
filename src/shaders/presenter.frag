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
    uvec2 position;
} camera;

layout(location = 0) out vec4 color;

void main() {
    vec2 positionScaled = position * vec2(uvec2(width, height));
    uvec2 index = uvec2(positionScaled);
    float value = float(1 - inputData.data[index.x + index.y * width]);
    if (camera.drawGrid == 1 && (fract(positionScaled.x) < 0.07 || fract(positionScaled.y) < 0.07 || fract(positionScaled.x) > 0.93 || fract(positionScaled.y) > 0.93)) {
        value = 0.9;
    } else if (index == camera.position) {
        value = value * 0.33 + 0.33;
    }
    color = vec4(value);
}