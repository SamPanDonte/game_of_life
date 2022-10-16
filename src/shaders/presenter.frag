#version 460 core

layout(location = 0) in vec2 position;

layout(set = 0, binding = 0) readonly buffer Data {
    uint data[];
} inputData;

layout(constant_id = 0) const uint width = 1024;
layout(constant_id = 1) const uint height = 1024;

layout(location = 0) out vec4 color;

void main() {
    uvec2 index = uvec2(position * vec2(uvec2(width, height)));
    color = vec4(inputData.data[index.x + index.y * width] == 1 ? 0.0 : 1.0);
}