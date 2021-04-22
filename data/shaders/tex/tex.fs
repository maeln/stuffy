#version 330 core
out vec4 FragColor;

in vec2 texCoords;

uniform float time;
uniform vec2 resolution;
uniform float frame_nb;

uniform sampler2D pathbuffer;
uniform sampler2D denoiserbuffer;

bool is_in(ivec2 coord) {
	return coord.x >= 0 && coord.x < resolution.x && coord.y >= 0 && coord.y < resolution.y;
}

vec3 blur(ivec2 coord, int n, float max_dist, float max_deviation, float ratio) {
	vec3 origin = texelFetch(denoiserbuffer, coord, 0).rgb;
	float deviation = 0.0;
	vec3 average = vec3(0.0);
	float valid = 0;

	for(int x = -n; x<=n; x++) {
		for(int y = -n; y<=n; y++) {
			if((x == 0 && y == 0) || !is_in(coord + ivec2(x,y))) {
				continue;
			}
			vec3 neighbor = texelFetch(denoiserbuffer, coord + ivec2(x,y), 0).rgb;
			float dist = distance(origin, neighbor);
			if(dist > max_dist) {
//				origin = vec3(1.0, 0.0, 0.0);
				continue;
			}
			deviation += dist;
			average += neighbor;
			valid += 1.0;
		}
	}

	if(valid < 1.0) {
		return origin;
	}

	deviation /= valid;
	if(deviation > max_deviation) {
//		origin = vec3(0.0, 1.0, 0.0);
		return origin;
	}

	average /= vec3(valid);
	vec3 direction = average - origin;
	direction *= vec3(ratio);

	return average;
}

void main()
{
	vec2 uv = gl_FragCoord.xy / resolution.xy;
	vec2 screenCoord = texCoords * resolution.xy;
	/*
	float iter = texelFetch(denoiserbuffer, ivec2(screenCoord), 0).a;
	vec3 one = blur(ivec2(screenCoord), 2, 0.8, 0.4, 1.0);
	vec3 two = blur(ivec2(screenCoord), 4, 0.4, 0.2, 0.6);
	vec3 three = blur(ivec2(screenCoord), 8, 0.2, 0.1, 0.4);
	FragColor = vec4((one+two+three)/vec3(3.0), iter);
	*/
	FragColor = texelFetch(denoiserbuffer, ivec2(screenCoord), 0);
}