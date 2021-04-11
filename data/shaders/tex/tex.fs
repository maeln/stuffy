#version 330 core
out vec4 FragColor;

in vec2 TexCoords;

uniform float time;
uniform vec2 resolution;
uniform float frame_nb;

uniform sampler2D backbuffer;
uniform sampler2D scenebuffer;

void main()
{
	vec2 uv = gl_FragCoord.xy / resolution.xy;
	FragColor = vec4(texture(scenebuffer, uv).rgb, 1.0);
}