#version 330 core
out vec4 FragColor;

in vec2 TexCoords;

uniform vec2 resolution;

uniform sampler2D tex;

void main()
{
	vec2 uv = gl_FragCoord.xy / resolution.xy;
	FragColor = vec4(texture(tex, uv).rgb, 1.0);
}