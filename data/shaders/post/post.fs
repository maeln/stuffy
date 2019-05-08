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

#define LAMBERTIAN 0
#define METAL 1
#define DIELECTRIC 2

#define SAMPLING 4

uint base_hash(uvec2 p) {
    p = 1103515245U*((p >> 1U)^(p.yx));
    uint h32 = 1103515245U*((p.x)^(p.y>>3U));
    return h32^(h32 >> 16);
}

float g_seed = 0.;

float hash1(inout float seed) {
    uint n = base_hash(floatBitsToUint(vec2(seed+=.1,seed+=.1)));
    return float(n)*(1.0/float(0xffffffffU));
}

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

struct material {
	vec3 albedo;
	float fuzz;
	float refraction;
	int type;
};

struct hit {
	float t;
	vec3 p;
	vec3 normal;
	material m;
};

struct sphere {
	float radius;
	vec3 center;
	material m;
};

ray new_ray(vec3 o, vec3 d) {
	ray r;
	r.origin = o;
	r.direction = d;
	return r;
}

hit new_hit(float ht, vec3 point, vec3 norm, material m) {
	hit h;
	h.t = ht;
	h.p = point;
	h.normal = norm;
	h.m = m;
	return h;
}

material new_material(vec3 a, int t, float f, float r) {
	material m;
	m.albedo = a;
	m.type = t;
	m.fuzz = f;
	m.refraction = r;
	return m;
}

sphere new_sphere(vec3 c, float r, material m) {
	sphere s;
	s.radius = r;
	s.center = c;
	s.m = m;
	return s;
}

vec3 point_at(ray r, float t) {
	return (r.origin + t * r.direction);
}

float schlick(float cosine, float ref_idx) {
	float r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
	r0 = r0*r0;
	return r0 + (1.0 - r0) * pow((1.0 - cosine), 5.0);
}

bool refraction(in vec3 dir, in vec3 normal, in float ni_over_nt, inout vec3 refracted) {
	float dt = dot(normalize(dir), normal);
	float discriminant = 1.0 - ni_over_nt*ni_over_nt * (1.0 - dt*dt);
	if(discriminant > 0.0) {
		refracted = ni_over_nt * (normalize(dir) - normal*dt) - normal*sqrt(discriminant);
		return true;
	}
	return false;
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
			h.m = s.m;
			return true;
		}

		dist = (-b + sqrt(discriminant)) / (2.0 * a);
		if(dist > t_min && t_max > dist) {
			h.t = dist;
			h.p = point_at(r, dist);
			h.normal = (h.p - s.center) / s.radius;
			h.m = s.m;
			return true;
		}
	}
	return false;
}

bool hit_scene(in ray r, in float t_min, in float t_max, out hit h) {
	sphere s1 = new_sphere(vec3(0.0, -100.5, -1.0), 100.0, new_material(vec3(0.1, 0.8, 0.3), METAL, 0.02, 0.0));
	sphere s2 = new_sphere(vec3(0.0, 0.0, -1.0), 0.5, new_material(vec3(0.8, 0.2, 0.2), LAMBERTIAN, 0.0, 0.0));
	sphere s3 = new_sphere(vec3(1.0, 0.0, -1.0), -0.5, new_material(vec3(0.0), DIELECTRIC, 0.0, 1.5));
	sphere s4 = new_sphere(vec3(-0.7, 0.0, 0.0), 0.5, new_material(vec3(1.0,  0.843, 0.0), METAL, 0.17, 0.0));
	
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

	if(hit_sphere(s3, r, t_min, closest, tmp_hit)) {
		closest = tmp_hit.t;
		got_hit = true;
		h = tmp_hit;
	}

	if(hit_sphere(s4, r, t_min, closest, tmp_hit)) {
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

bool lambertian_scatter(in ray r, in hit h, out vec3 attenuation, out ray scattered) {
	vec3 target = h.normal + random_in_unit_sphere(g_seed);
	scattered = new_ray(h.p, target);
	attenuation = h.m.albedo;
	return true;
}

bool metal_scatter(in ray r, in hit h, out vec3 attenuation, inout ray scattered) {
	vec3 reflected = reflect(normalize(r.direction), h.normal);
	scattered = new_ray(h.p, reflected + h.m.fuzz * random_in_unit_sphere(g_seed));
	attenuation = h.m.albedo;
	return (dot(scattered.direction, h.normal) > 0.0);
}

bool dielectric_scatter(in ray r, in hit h, out vec3 attenuation, inout ray scattered) {
	vec3 outward_normal;
	vec3 reflected = reflect(normalize(r.direction), h.normal);
	float ni_over_nt;
	attenuation = vec3(1.0);
	vec3 refracted;
	float reflect_prob;
	float cosine;

	if(dot(normalize(r.direction), h.normal) > 0.0) {
		outward_normal = -h.normal;
		ni_over_nt = h.m.refraction;
		cosine = h.m.refraction * dot(normalize(r.direction), h.normal) / length(r.direction);
	} else {
		outward_normal = h.normal;
		ni_over_nt = 1.0 / h.m.refraction;
		cosine = -dot(normalize(r.direction), h.normal) / length(r.direction);
	}

	if(refraction(r.direction, h.normal, ni_over_nt, refracted)) {
		reflect_prob = schlick(cosine, h.m.refraction);
	} else {
		reflect_prob = 1.0;
	}

	if(hash1(g_seed) < reflect_prob) {
		scattered = ray(h.p, reflected);
	} else {
		scattered = ray(h.p, refracted);
	}

	return true;
}

bool material_scatter(in ray r, in hit h, out vec3 attenuation, inout ray scattered) {
	if(h.m.type == LAMBERTIAN) {
		return lambertian_scatter(r, h, attenuation, scattered);
	} else if(h.m.type == METAL) {
		return metal_scatter(r, h, attenuation, scattered);
	} else if(h.m.type == DIELECTRIC) {
		return dielectric_scatter(r, h, attenuation, scattered);
	} else {
		return false;
	}
}

vec3 color(in ray r) {
	vec3 c = vec3(1.0);
	hit h;

	for(int i=0; i<MAX_BOUNCE; ++i) {
		if(hit_scene(r, T_MIN, T_MAX, h)) {
			ray scattered;
			vec3 attenuation;
			if(material_scatter(r, h, attenuation, scattered)) {
				c *= attenuation;
				r = scattered;
			} else {
				return vec3(0.0);
			}
		} else {
			c *= sky(r);
			return c;
		}
	}

	return c;
}

ray get_cam_ray(vec3 eye, vec3 lookat, vec3 up, float vfov, float aspect, vec2 uv) {
	float theta = vfov * PI / 180.0;
	float half_height = tan(theta/2.0);
	float half_width = aspect * half_height;
	vec3 w = normalize(eye - lookat);
	vec3 u = normalize(cross(up, w));
	vec3 v = cross(w, u);
	
	vec3 lower_left_corner = eye - half_width*u - half_height*v -w;
	vec3 horizontal = 2.0*half_width*u;
	vec3 vertical = 2.0*half_height*v;
	

	vec2 jitter = hash2(g_seed) / resolution.xy;
	return new_ray(eye, lower_left_corner + (uv.x+jitter.x) * horizontal + (uv.y+jitter.y) * vertical);
}

void main()
{
	vec2 uv = gl_FragCoord.xy / resolution.xy;

	// Init the seed. Make it different for each pixel + frame.
	g_seed = float(base_hash(floatBitsToUint(gl_FragCoord.xy)))/float(0xffffffffU)+time;

	vec3 col = vec3(0.0);
	for(int i=0; i<SAMPLING; ++i) {
		ray r = get_cam_ray(vec3(0.0, 0.0, -1.0), vec3(0.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0), 90.0, resolution.x/resolution.y, uv);
		col += color(r);
	}

	col /= float(SAMPLING);

	vec3 previous = texture(backbuffer, uv).rgb * frame_nb;
	previous += col;
	previous /= (frame_nb + 1.0);

	FragColor = vec4(previous, 1.0);
}