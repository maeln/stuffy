#version 330 core
out vec4 FragColor;

in vec2 TexCoords;

uniform float time;
uniform vec2 resolution;
uniform float frame_nb;
uniform sampler2D backbuffer;

#define PI 3.141592
#define saturate(x) (clamp((x), 0.0, 1.0))

#define T_MIN 1e-4
#define T_MAX 100.0
#define MAX_BOUNCE 5

#define SAMPLING 4

uint base_hash(uvec2 p) {
    p = 1103515245U*((p >> 1U)^(p.yx));
    uint h32 = 1103515245U*((p.x)^(p.y>>3U));
    return h32^(h32 >> 16);
}

float g_seed = 0.;

vec2 hash2(inout float seed) {
    uint n = base_hash(floatBitsToUint(vec2(seed+=.1,seed+=.1)));
    uvec2 rz = uvec2(n, n*48271U);
    return vec2(rz.xy & uvec2(0x7fffffffU))/float(0x7fffffff);
}

vec3 hash3(inout float seed) {
    uint n = base_hash(floatBitsToUint(vec2(seed+=.1,seed+=.1)));
    uvec3 rz = uvec3(n, n*16807U, n*48271U);
    return vec3(rz & uvec3(0x7fffffffU))/float(0x7fffffff);
}

/*
vec3 random_in_unit_sphere(in float seed) {
	vec3 dir = vec3(rand(seed), rand(seed*2.0), rand(seed*3.0));
	float len = rand(seed*seed);
	return dir*len;
}
*/
vec3 random_in_unit_sphere(inout float seed) {
    vec3 h = hash3(seed) * vec3(2.,6.28318530718,1.)-vec3(1,0,0);
    float phi = h.y;
    float r = pow(h.z, 1./3.);
	return r * vec3(sqrt(1.-h.x*h.x)*vec2(sin(phi),cos(phi)),h.x);
}


struct ray {
	vec3 origin;
	vec3 direction;
};

struct hit {
	float t;
	vec3 p;
	vec3 normal;
};

struct sphere {
	float radius;
	vec3 center;
};

ray new_ray(vec3 o, vec3 d) {
	ray r;
	r.origin = o;
	r.direction = d;
	return r;
}

hit new_hit(float ht, vec3 point, vec3 norm) {
	hit h;
	h.t = ht;
	h.p = point;
	h.normal = norm;
	return h;
}

sphere new_sphere(vec3 c, float r) {
	sphere s;
	s.radius = r;
	s.center = c;
	return s;
}

vec3 point_at(ray r, float t) {
	return (r.origin + t * r.direction);
}

bool hit_sphere(in sphere s, in ray r, in float t_min, in float t_max, out hit h) {
	vec3 oc = r.origin - s.center;
	float a = dot(r.direction, r.direction);
	float b = 2.0 * dot(oc, r.direction);
	float c = dot(oc, oc) - s.radius*s.radius;
	float discriminant = b*b - 4*a*c;
	if(discriminant > 0.0) {
		float dist = (-b - sqrt(discriminant)) / (2.0 * a);
		if(dist > t_min && t_max > dist) {
			h.t = dist;
			h.p = point_at(r, dist);
			h.normal = (h.p - s.center) / s.radius;
			return true;
		}

		dist = (-b + sqrt(discriminant)) / (2.0 * a);
		if(dist > t_min && t_max > dist) {
			h.t = dist;
			h.p = point_at(r, dist);
			h.normal = (h.p - s.center) / s.radius;
			return true;
		}
	}
	return false;
}

bool hit_scene(in ray r, in float t_min, in float t_max, out hit h) {
	sphere s1 = new_sphere(vec3(0.0, 0.0, -1.0), 0.5);
	sphere s2 = new_sphere(vec3(0.0, -100.5, -1.0), 100.0);
	
	hit tmp_hit;
	float closest = t_max;
	bool got_hit = false;

	if(hit_sphere(s1, r, t_min, closest, tmp_hit)) {
		closest = tmp_hit.t;
		got_hit = true;
		h = tmp_hit;
	}

	if(hit_sphere(s2, r, t_min, closest, tmp_hit)) {
		closest = tmp_hit.t;
		got_hit = true;
		h = tmp_hit;
	}

	return got_hit;
}

vec3 sky(in ray r) {
	vec3 unit_dir = normalize(r.direction);
	float t = 0.5 * (unit_dir.y + 1.0);
	return (1.0-t) * vec3(1.0) + t * vec3(0.5, 0.7, 1.0);
}

vec3 color(in ray r) {
	vec3 c = vec3(1.0);
	hit h;

	for(int i=0; i<MAX_BOUNCE; ++i) {
		if(hit_scene(r, T_MIN, T_MAX, h)) {
			c *= 0.5;
			r.origin = h.p;
			r.direction = h.normal + random_in_unit_sphere(g_seed);
		} else {
			c *= sky(r);
			return c;
		}
	}

	return c;
}

vec3 lower_left = vec3(-2.0, -1.0, -1.0);
vec3 horizontal = vec3(4.0, 0.0, 0.0);
vec3 vertical = vec3(0.0, 2.0, 0.0);
vec3 origin = vec3(0.0, 0.0, 0.0);

void main()
{
	vec2 uv = gl_FragCoord.xy / resolution.xy;

	// Init the seed. Make it different for each pixel + frame.
	g_seed = float(base_hash(floatBitsToUint(gl_FragCoord.xy)))/float(0xffffffffU)+time;

	vec3 col = vec3(0.0);
	for(int i=0; i<SAMPLING; ++i) {
		vec2 jitter = hash2(g_seed) / resolution.xy;
		ray r = new_ray(origin, lower_left + (uv.x+jitter.x) * horizontal + (uv.y+jitter.y) * vertical);
		col += color(r);
	}

	col /= float(SAMPLING);

	vec3 previous = texture(backbuffer, uv).rgb * frame_nb;
	previous += col;
	previous /= (frame_nb + 1.0);

	FragColor = vec4(previous, 1.0);
}