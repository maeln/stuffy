#version 330 core
out vec4 FragColor;

in vec2 TexCoords;

uniform float time;
uniform vec2 resolution;

#define PI 3.141592
#define saturate(x) (clamp((x), 0.0, 1.0))

#define T_MIN 0.0
#define T_MAX 100.0

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

vec3 color(ray r) {
	hit h;
	if(hit_scene(r, T_MIN, T_MAX, h)) {
		return 0.5 * vec3(h.normal.x+1.0, h.normal.y+1.0, h.normal.z+1.0);
	} else {
		vec3 unit_dir = normalize(r.direction);
		float t = 0.5 * (unit_dir.y + 1.0);
		return (1.0-t) * vec3(1.0) + t * vec3(0.5, 0.7, 1.0);
	}	
}

vec3 lower_left = vec3(-2.0, -1.0, -1.0);
vec3 horizontal = vec3(4.0, 0.0, 0.0);
vec3 vertical = vec3(0.0, 2.0, 0.0);
vec3 origin = vec3(0.0, 0.0, 0.0);

void main()
{
	vec2 uv = gl_FragCoord.xy / resolution.xy;
	ray r = new_ray(origin, lower_left + uv.x * horizontal + uv.y * vertical);
	vec3 col = color(r);

	FragColor = vec4(col, 1.0);
}