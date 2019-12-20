#version 100

attribute vec2 position;
attribute vec2 texcoord;

varying lowp vec4 v_texcoord;

void main()
{
    gl_Position = vec4(position, 0.0, 1.0);
    v_texcoord = vec4(texcoord, 0.0, 0.0);
}
