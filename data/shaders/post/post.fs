#version 330 core
out vec4 FragColor;

in vec2 TexCoords;

uniform float time;
uniform vec2 resolution;

#define PI 3.141592
#define saturate(x) (clamp((x), 0.0, 1.0))

void main()
{
	vec2 uv = gl_FragCoord.xy / resolution.xy;

	vec3 col = 0.5 + 0.5*cos(time/1000.0+uv.xyx+vec3(0,2,4));

	FragColor = vec4(col, 1.0);
}