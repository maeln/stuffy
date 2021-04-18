#version 410
out vec4 FragColor;

in vec2 TexCoords;

uniform float time;
uniform vec2 resolution;
uniform float frame_nb;
//uniform sampler2D backbuffer;
//uniform sampler2D scenebuffer;

#define PI 3.141592
#define saturate(x) (clamp((x), 0.0, 1.0))

#define T_MIN 1e-5
#define T_MAX 100.0
#define MAX_BOUNCE 8

#define LAMBERTIAN 0
#define METAL 1
#define DIELECTRIC 2

#define SAMPLING 4

float g_seed = 0.0;

uint base_hash(uvec2 p) {
  p = 1103515245U * ((p >> 1U) ^ (p.yx));
  uint h32 = 1103515245U * ((p.x) ^ (p.y >> 3U));
  return h32 ^ (h32 >> 16);
}

/*
float hash1(inout float seed) {
  uint n = base_hash(floatBitsToUint(vec2(seed += .1, seed += .1)));
  return float(n) * (1.0 / float(0xffffffffU));
}

vec2 hash2(inout float seed) {
  uint n = base_hash(floatBitsToUint(vec2(seed += .1, seed += .1)));
  uvec2 rz = uvec2(n, n * 48271U);
  return vec2(rz.xy & uvec2(0x7fffffffU)) / float(0x7fffffff);
}

vec3 hash3(inout float seed) {
  uint n = base_hash(floatBitsToUint(vec2(seed += .1, seed += .1)));
  uvec3 rz = uvec3(n, n * 16807U, n * 48271U);
  return vec3(rz & uvec3(0x7fffffffU)) / float(0x7fffffff);
}
*/

lowp float hash1(inout float seed) {
    return fract(sin(seed += 0.1)*43758.5453123);
}

lowp vec2 hash2(inout float seed) {
    return fract(sin(vec2(seed+=0.1,seed+=0.1))*vec2(43758.5453123,22578.1459123));
}

lowp vec3 hash3(inout float seed) {
    return fract(sin(vec3(seed+=0.1,seed+=0.1,seed+=0.1))*vec3(43758.5453123,22578.1459123,19642.3490423));
}

/*
vec3 random_in_unit_sphere(in float seed) {
        vec3 dir = vec3(rand(seed), rand(seed*2.0), rand(seed*3.0));
        float len = rand(seed*seed);
        return dir*len;
}
*/
vec3 random_in_unit_sphere(inout float seed) {
  vec3 h = hash3(seed) * vec3(2., 6.28318530718, 1.) - vec3(1, 0, 0);
  float phi = h.y;
  float r = pow(h.z, 1. / 3.);
  return r * vec3(sqrt(1. - h.x * h.x) * vec2(sin(phi), cos(phi)), h.x);
}

vec3 random_in_unit_disk() {
    while (true) {
        vec3 p = vec3(hash1(g_seed), hash1(g_seed), 0);
        if (dot(p,p) >= 1) continue;
        return p;
    }
}

struct ray {
  vec3 origin;
  vec3 direction;
};

struct material {
  vec3 albedo;
  vec3 emission;
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
  m.emission = vec3(0.0);
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

vec3 point_at(ray r, float t) { return (r.origin + t * r.direction); }

float schlick(float cosine, float ref_idx) {
  float r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
  r0 = r0 * r0;
  return r0 + (1.0 - r0) * pow((1.0 - cosine), 5.0);
}

bool refraction(in vec3 dir, in vec3 normal, in float ni_over_nt,
                inout vec3 refracted) {
  float dt = dot(normalize(dir), normal);
  float discriminant = 1.0 - ni_over_nt * ni_over_nt * (1.0 - dt * dt);
  if (discriminant > 0.0) {
    refracted = ni_over_nt * (normalize(dir) - normal * dt) -
                normal * sqrt(discriminant);
    return true;
  }
  return false;
}

bool modified_refract(const in vec3 v, const in vec3 n,
                      const in float ni_over_nt, out vec3 refracted) {
  float dt = dot(v, n);
  float discriminant = 1. - ni_over_nt * ni_over_nt * (1. - dt * dt);
  if (discriminant > 0.) {
    refracted = ni_over_nt * (v - n * dt) - n * sqrt(discriminant);
    return true;
  } else {
    return false;
  }
}

bool hit_sphere(in sphere s, in ray r, in float t_min, in float t_max,
                out hit h) {
  vec3 oc = r.origin - s.center;
  float a = dot(r.direction, r.direction);
  float b = 2.0 * dot(oc, r.direction);
  float c = dot(oc, oc) - s.radius * s.radius;
  float discriminant = b * b - 4 * a * c;
  if (discriminant > 0.0) {
    float dist = (-b - sqrt(discriminant)) / (2.0 * a);
    if (dist > t_min && t_max > dist) {
      h.t = dist;
      h.p = point_at(r, dist);
      h.normal = (h.p - s.center) / s.radius;
      h.m = s.m;
      return true;
    }

    dist = (-b + sqrt(discriminant)) / (2.0 * a);
    if (dist > t_min && t_max > dist) {
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
  sphere s1 =
      new_sphere(vec3(0.0, -100.5, -1.0), 100.0,
                 new_material(vec3(1.0), LAMBERTIAN, 0.0, 0.0));
  
  sphere s2 = new_sphere(vec3(0.0, 5.0, 2.0), 0.4,
                         new_material(vec3(0.0), LAMBERTIAN, 0.0, 0.0));
  s2.m.emission = vec3(1.0)*30.0;

  sphere s3 =
      new_sphere(vec3(0.0, 0.3, 0.0), 0.5,
                 new_material(vec3(0.1, 0.2, 0.5), LAMBERTIAN, 0.3, 0.0));
  // s3.m.emission = vec3(1.0)*20.0;

  sphere s4 = new_sphere(vec3(2.0, 1.0, -1.0), 0.5,
                         new_material(vec3(0.8, 0.6, 0.2), METAL, 0.00, 0.0));
  
  sphere s5 = new_sphere(vec3(-3.0, 0.6, 2.0), 1.0,
                         new_material(vec3(0.0), DIELECTRIC, 0.0, 1.03));
  
  sphere s6 = new_sphere(vec3(2.0, 1.2, -1.0), 1.5,
                         new_material(vec3(0.0), DIELECTRIC, 0.0, 0.95));

  sphere s7 =
      new_sphere(vec3(-5.0, 1.0, 0.0), 1.5,
                 new_material(vec3(0.8, 0.2, 0.5), LAMBERTIAN, 0.3, 0.0));

  hit tmp_hit;
  float closest = t_max;
  bool got_hit = false;

  if (hit_sphere(s1, r, t_min, closest, tmp_hit)) {
    closest = tmp_hit.t;
    got_hit = true;
    h = tmp_hit;
  }
  
  if (hit_sphere(s2, r, t_min, closest, tmp_hit)) {
    closest = tmp_hit.t;
    got_hit = true;
    h = tmp_hit;
  } else if (hit_sphere(s3, r, t_min, closest, tmp_hit)) {
    closest = tmp_hit.t;
    got_hit = true;
    h = tmp_hit;
  } else if (hit_sphere(s4, r, t_min, closest, tmp_hit)) {
    closest = tmp_hit.t;
    got_hit = true;
    h = tmp_hit;
  } else if (hit_sphere(s5, r, t_min, closest, tmp_hit)) {
    closest = tmp_hit.t;
    got_hit = true;
    h = tmp_hit;
  } else if (hit_sphere(s6, r, t_min, closest, tmp_hit)) {
    closest = tmp_hit.t;
    got_hit = true;
    h = tmp_hit;
  } else if (hit_sphere(s7, r, t_min, closest, tmp_hit)) {
    closest = tmp_hit.t;
    got_hit = true;
    h = tmp_hit;
  }

  return got_hit;
}

vec3 sky(in ray r) {
  vec3 unit_dir = normalize(r.direction);
  float t = 0.5 * (unit_dir.y + 1.0);
  return ((1.0 - t) * vec3(0.7, 0.3, 0.3) + t * vec3(0.8, 0.2, 0.3)) * 0.05;
}

bool lambertian_scatter(in ray r, in hit h, out vec3 attenuation,
                        out ray scattered) {
  vec3 target = h.normal + random_in_unit_sphere(g_seed);
  scattered = new_ray(h.p, target);
  attenuation = h.m.albedo;
  return true;
}

bool metal_scatter(in ray r, in hit h, out vec3 attenuation,
                   inout ray scattered) {
  vec3 reflected = reflect(normalize(r.direction), h.normal);
  scattered =
      new_ray(h.p, reflected + h.m.fuzz * random_in_unit_sphere(g_seed));
  attenuation = h.m.albedo;
  return (dot(scattered.direction, h.normal) > 0.0);
}

bool dielectric_scatter(in ray r, in hit h, out vec3 attenuation,
                        inout ray scattered) {
  vec3 outward_normal, refracted;
  vec3 reflected = reflect(r.direction, h.normal);
  float ni_over_nt, reflect_prob, cosine;
  attenuation = vec3(1.0);

  if (dot(r.direction, h.normal) > 0.0) {
    outward_normal = -h.normal;
    ni_over_nt = h.m.refraction;
    cosine = h.m.refraction * dot(normalize(r.direction), h.normal) /
             length(r.direction);
  } else {
    outward_normal = h.normal;
    ni_over_nt = 1.0 / h.m.refraction;
    cosine = -dot(r.direction, h.normal);
  }

  if (refraction(r.direction, outward_normal, ni_over_nt, refracted)) {
    reflect_prob = schlick(cosine, h.m.refraction);
  } else {
    reflect_prob = 1.0;
  }

  if (hash1(g_seed) < reflect_prob) {
    scattered = ray(h.p, reflected);
  } else {
    scattered = ray(h.p, refracted);
  }

  return true;
}

bool material_scatter(in ray r, in hit h, out vec3 attenuation,
                      inout ray scattered) {
  if (h.m.type == LAMBERTIAN) {
    return lambertian_scatter(r, h, attenuation, scattered);
  } else if (h.m.type == METAL) {
    return metal_scatter(r, h, attenuation, scattered);
  } else if (h.m.type == DIELECTRIC) {
    return dielectric_scatter(r, h, attenuation, scattered);
  } else {
    return false;
  }
}

vec3 color(in ray r) {
  vec3 ret = vec3(0.0);
  vec3 c = vec3(1.0);
  hit h;

  for (int i = 0; i < MAX_BOUNCE; ++i) {
    if (hit_scene(r, T_MIN, T_MAX, h)) {
      ray scattered;
      vec3 attenuation;
      if (material_scatter(r, h, attenuation, scattered)) {
        ret += h.m.emission * c;
        c *= attenuation * c;
        r = scattered;
      } else {
        return h.m.emission;
      }
    } else {
      ret += sky(r) * c;
      return ret;
    }
  }

  return ret;
}

ray get_cam_ray(vec3 eye, vec3 lookat, vec3 up, float vfov, float aspect,
                vec2 uv, float aperture, float focus_dist) {
  float theta = vfov * PI / 180.0;
  float half_height = tan(theta / 2.0);
  float half_width = aspect * half_height;
  vec3 w = normalize(eye - lookat);
  vec3 u = normalize(cross(up, w));
  vec3 v = cross(w, u);

  vec3 horizontal = focus_dist * half_width * u;
  vec3 vertical = focus_dist * half_height * v;
  vec3 lower_left_corner = eye - horizontal/2.0 - vertical/2.0 - focus_dist*w;

  vec2 jitter = hash2(g_seed) / resolution.xy;
  float lens_radius = aperture / 2.0;

  vec3 rd = lens_radius * random_in_unit_disk();
  vec3 offset = u * rd.x + v * rd.y;

  return new_ray(eye + offset, 
                  lower_left_corner + (uv.x + jitter.x) * horizontal + (uv.y + jitter.y) * vertical - eye - offset);
}

void main() {
  vec2 uv = gl_FragCoord.xy / resolution.xy;

  // Init the seed. Make it different for each pixel + frame.
  g_seed = float(base_hash(floatBitsToUint(gl_FragCoord.xy))) / float(0xffffffffU) + time;

  vec3 eye = vec3(7.0, 2.5, 5.0);
  vec3 lookat = vec3(0.0, 0.5, 0.0);
  float aspect = resolution.x / resolution.y;
  float focus_dist = distance(eye, lookat);

  vec3 col = vec3(0.0);
  for (int i = 0; i < SAMPLING; ++i) {
    ray r = get_cam_ray(eye, lookat, vec3(0.0, 1.0, 0.0), 90.0, aspect, uv, 0.7, focus_dist);
    col += color(r);
  }

  col /= float(SAMPLING);

  FragColor = vec4(col, 1.0);
}