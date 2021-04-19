#version 330 core
out vec4 FragColor;

in vec2 texCoords;

uniform float time;
uniform vec2 resolution;
uniform float frame_nb;

uniform sampler2D pathbuffer;

bool is_in(ivec2 coord) {
	return coord.x >= 0 && coord.x < resolution.x && coord.y >= 0 && coord.y < resolution.y;
}

vec4 merge4x4(ivec2 coord) {
	vec4 origin = texelFetch(pathbuffer, coord, 0);
	for(int x = -2; x<=2; x++) {
		for(int y = -2; y<=2; y++) {
			if(x == 0 && y ==0 && is_in(coord + ivec2(x,y))) {
				continue;
			}
			origin += texelFetch(pathbuffer, coord + ivec2(x,y), 0);
		}
	}
	return origin / vec4(4.0);
}

void main()
{
	vec2 uv = gl_FragCoord.xy / resolution.xy;
	vec2 screenCoord = texCoords * resolution.xy;
	// FragColor = texelFetch(pathbuffer, ivec2(screenCoord), 0);
	FragColor = merge4x4(ivec2(screenCoord));
}