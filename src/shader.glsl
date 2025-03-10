-- vertex
#version 330 core

layout (location = 0) in vec2 position;
layout (location = 1) in vec2 i_tex_coord;

out vec2 tex_coord;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    tex_coord = i_tex_coord;
}

-- fragment
#version 330 core

in vec2 tex_coord;
out vec4 frag_color;

uniform sampler2D tex;

void main() {
    frag_color = texture(tex, tex_coord);
}
