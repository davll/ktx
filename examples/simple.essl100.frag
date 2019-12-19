#version 100

uniform sampler2D tex;

varying lowp vec4 v_texcoord;

void main()
{
    gl_FragColor = texture2D(tex, v_texcoord.st);
    //gl_FragColor = vec4(0.0, 0.5, 0.0, 1.0);
}
