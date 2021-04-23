#version 410
out vec4 FragColor;

in vec2 TexCoords;

uniform float time;
uniform vec2 resolution;
uniform float frame_nb;
//uniform sampler2D backbuffer;
//uniform sampler2D scenebuffer;

uniform vec3 in_eye;
uniform vec3 in_target;
uniform vec3 in_up;
uniform vec2 in_focus_pos;
uniform float in_aperture;

#define PI 3.141592
#define saturate(x) (clamp((x), 0.0, 1.0))

#define T_MIN 1e-6
#define T_MAX 100.0
#define MAX_BOUNCE 8

#define LAMBERTIAN 0
#define METAL 1
#define DIELECTRIC 2
#define VOLUME 3
#define EMISSIVE 99

#define SAMPLING 4

#define L_POS vec3(0.0, 5.0, 0.0)

float g_seed = 0.0;

uint base_hash(uvec2 p) {
  p = 1103515245U * ((p >> 1U) ^ (p.yx));
  uint h32 = 1103515245U * ((p.x) ^ (p.y >> 3U));
  return h32 ^ (h32 >> 16);
}

lowp float hash1(inout float seed) {
    return fract(sin(seed += 0.1)*43758.5453123);
}

lowp vec2 hash2(inout float seed) {
    return fract(sin(vec2(seed+=0.1,seed+=0.1))*vec2(43758.5453123,22578.1459123));
}

lowp vec3 hash3(inout float seed) {
    return fract(sin(vec3(seed+=0.1,seed+=0.1,seed+=0.1))*vec3(43758.5453123,22578.1459123,19642.3490423));
}

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

vec3 random_in_hemisphere(vec3 normal) {
  vec3 in_unit_sphere = random_in_unit_sphere(g_seed);
  if(dot(in_unit_sphere, normal) > 0.0) {
    return in_unit_sphere;
  } else {
    return - in_unit_sphere;
  }
}

bool near_zero(vec3 v) {
  vec3 z = vec3(1e-8);
  return (v.x < z.x) && (v.y < z.y) && (v.z < z.z);
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
  bool front_face;
  material m;
};

struct sphere {
  float radius;
  vec3 center;
  material m;
};

struct rect {
  vec4 pos;
  float k;
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

void set_face_normal(inout hit h, in ray r, in vec3 outward_normal) {
  h.front_face = dot(r.direction, outward_normal) < 0.0;
  h.normal = h.front_face ? outward_normal : -outward_normal;
}

bool hit_rect_xy(in rect m_rect,  in ray r, in float t_min, in float t_max, out hit h) {
  float t = (m_rect.k - r.origin.z) / r.direction.z;
  if(t < t_min || t > t_max) {
    return false;
  }

  float x = r.origin.x + t*r.direction.x;
  float y = r.origin.y + t*r.direction.y;
  if(x < m_rect.pos.x || x > m_rect.pos.y || y < m_rect.pos.z || y > m_rect.pos.w) {
    return false;
  }

  h.t = t;
  vec3 outward_normal = vec3(0.0, 0.0, 1.0);
  set_face_normal(h, r, outward_normal);
  h.m = m_rect.m;
  h.p = point_at(r, t);

  return true;
}

bool hit_sphere(in sphere s, in ray r, in float t_min, in float t_max, out hit h) {
  vec3 oc = r.origin - s.center;
  float a = dot(r.direction, r.direction);
  float half_b = dot(oc, r.direction);
  float c = dot(oc, oc) - s.radius * s.radius;
  float discriminant = half_b * half_b - a * c;
  if(discriminant < 0.0) {
    return false;
  }
  
  float sqrtd = sqrt(discriminant);
  float root = (-half_b - sqrtd) / a;
  if(root < t_min || t_max < root) {
    root = (-half_b + sqrtd) / a;
    if(root < t_min || t_max < root) {
      return false;
    }
  }

  h.t = root;
  h.p = point_at(r, root);
  vec3 outward_normal = (h.p - s.center) / s.radius;
  set_face_normal(h, r, outward_normal);
  h.m = s.m;

  return true;
}

bool volume_sphere_hit(in sphere m_sphere, in ray r, in float t_min, in float t_max, out hit h, float neg_inv_density) {
  hit r1;
  hit r2;
  if(!hit_sphere(m_sphere, r, -1e6, 1e6, r1)) {
    return false;
  }
  if(!hit_sphere(m_sphere, r, r1.t+0.001, 1e6, r2)) {
    return false;
  }

  if(r1.t < t_min) {
    r1.t = t_min;
  }
  if(r2.t > t_max) {
    r2.t = t_max;
  }

  if(r1.t >= r2.t) {
    return false;
  }

  if(r1.t < 0) {
    r1.t = 0;
  }

  float len = length(r.direction);
  float dist_in_bound = (r2.t - r1.t)*len;
  float hit_dist = -(1.0/neg_inv_density) * log(hash1(g_seed));
  if(hit_dist > dist_in_bound) {
    return false;
  }

  h.t = r1.t + hit_dist / len;
  h.p = point_at(r, h.t);
  h.normal = vec3(1.0, 0.0, 0.0);
  h.front_face = true;
  h.m = m_sphere.m;

  return true;
}

bool hit_scene(in ray r, in float t_min, in float t_max, out hit h) {
  sphere s1 =
      new_sphere(vec3(0.0, -100.5, 0.0), 100.0,
                 new_material(vec3(1.0), LAMBERTIAN, 0.0, 0.0));
  
  sphere s2 = new_sphere(L_POS, 1.0, new_material(vec3(0.0), EMISSIVE, 0.0, 0.0));
  s2.m.emission = vec3(5.0);

  sphere s3 = new_sphere(vec3(-0.3, 1.3, -1.3), 1.0, new_material(vec3(0.1, 0.2, 0.5), LAMBERTIAN, 0.0, 0.0));
  // s3.m.emission = vec3(10.0);

  sphere s4 = new_sphere(vec3(2.0, 1.0, -1.0), 0.5,
                         new_material(vec3(0.8, 0.6, 0.2), METAL, 0.00, 0.0));
  
  sphere s5 = new_sphere(vec3(-3.0, 0.6, 2.0), 1.0,
                         new_material(vec3(0.0), DIELECTRIC, 0.0, 1.06));
  
  sphere s6 = new_sphere(vec3(2.0, 1.2, -1.0), -1.5,
                         new_material(vec3(0.0), DIELECTRIC, 0.0, 0.95));

  sphere s7 =
      new_sphere(vec3(-5.0, 1.0, 0.0), 1.5,
                 new_material(vec3(0.8, 0.2, 0.5), LAMBERTIAN, 0.0, 0.0));
  
  sphere s8 = new_sphere(vec3(0.0, 0.0, 0.0), 100.5, new_material(vec3(1.0), VOLUME, 0.0, 0.0));

  rect r1;
  r1.pos = vec4(3.0, 5.0, 1.0, 3.0);
  r1.k = -2.0;
  r1.m = new_material(vec3(0.2, 0.8, 0.3), LAMBERTIAN, 0.0, 0.0);

  hit tmp_hit;
  float closest = t_max;
  bool got_hit = false;

  if(volume_sphere_hit(s8, r, t_min, closest, tmp_hit, 0.025)) {
    closest = tmp_hit.t;
    got_hit = true;
    h = tmp_hit;
  }
  if (hit_sphere(s1, r, t_min, closest, tmp_hit)) {
    closest = tmp_hit.t;
    got_hit = true;
    h = tmp_hit;
  } 
  if (hit_sphere(s2, r, t_min, closest, tmp_hit)) {
    closest = tmp_hit.t;
    got_hit = true;
    h = tmp_hit;
  }  
  if (hit_sphere(s3, r, t_min, closest, tmp_hit)) {
    closest = tmp_hit.t;
    got_hit = true;
    h = tmp_hit;
  }  
  if (hit_sphere(s4, r, t_min, closest, tmp_hit)) {
    closest = tmp_hit.t;
    got_hit = true;
    h = tmp_hit;
  }  
  if (hit_sphere(s5, r, t_min, closest, tmp_hit)) {
    closest = tmp_hit.t;
    got_hit = true;
    h = tmp_hit;
  }  
  if (hit_sphere(s6, r, t_min, closest, tmp_hit)) {
    closest = tmp_hit.t;
    got_hit = true;
    h = tmp_hit;
  }  
  if (hit_sphere(s7, r, t_min, closest, tmp_hit)) {
    closest = tmp_hit.t;
    got_hit = true;
    h = tmp_hit;
  } 
  if (hit_rect_xy(r1, r, t_min, closest, tmp_hit)) {
    closest = tmp_hit.t;
    got_hit = true;
    h = tmp_hit;
  }
  

  return got_hit;
}

vec3 sky(in ray r) {
  vec3 unit_dir = normalize(r.direction);
  float t = 0.5 * (unit_dir.y + 1.0);
  //return ((1.0 - t) * vec3(0.95, 0.5, 0.5) + t * vec3(0.8, 0.5, 0.6)) * 0.25;
  return vec3(0.00);
}

bool lambertian_scatter(in ray r, in hit h, out vec3 attenuation, out ray scattered) {
  vec3 new_dir = h.normal + random_in_hemisphere(h.normal);
  if(near_zero(new_dir)) {
    new_dir = h.normal;
  }
  scattered = new_ray(h.p, new_dir);
  attenuation = h.m.albedo;
  return true;
}

bool metal_scatter(in ray r, in hit h, out vec3 attenuation, inout ray scattered) {
  vec3 reflected = reflect(normalize(r.direction), h.normal);
  scattered =
      new_ray(h.p, reflected + h.m.fuzz * random_in_unit_sphere(g_seed));
  attenuation = h.m.albedo;
  return (dot(scattered.direction, h.normal) > 0.0);
}

bool dielectric_scatter(in ray r, in hit h, out vec3 attenuation, inout ray scattered) {
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

bool isotropic_scatter(in ray r, in hit h, out vec3 attenuation, inout ray scattered) {
  scattered = ray(h.p, random_in_unit_sphere(g_seed));
  attenuation = h.m.albedo;
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
  } else if (h.m.type == VOLUME) {
    return isotropic_scatter(r, h, attenuation, scattered);
  }
  return false;
}

vec3 ortho(vec3 v) {
    return abs(v.x) > abs(v.z) ? vec3(-v.y, v.x, 0.0)  : vec3(0.0, -v.z, v.y);
}

vec3 getConeSample(vec3 dir, float extent) {
        // Formula 34 in GI Compendium
	dir = normalize(dir);
	vec3 o1 = normalize(ortho(dir));
	vec3 o2 = normalize(cross(dir, o1));
	vec2 r =  hash2(g_seed);
	r.x=r.x*2.*PI;
	r.y=1.0-r.y*extent;
	float oneminus = sqrt(1.0-r.y*r.y);
	return cos(r.x)*oneminus*o1+sin(r.x)*oneminus*o2+r.y*dir;
}

vec3 color(in ray r) {
  vec3 col = vec3(0.0);
  vec3 emitted = vec3(0.0);
  hit h;

  for (int i = 0; i < MAX_BOUNCE; ++i) {
    if (hit_scene(r, T_MIN, T_MAX, h)) {
      ray scattered;
      vec3 attenuation;
      vec3 emit = h.m.emission;
      emitted += i == 0 ? emit : col * emit;

      if (material_scatter(r, h, attenuation, scattered)) {
        float russian = max(0.05, 3.0 - length(attenuation));
        if(hash1(g_seed) > russian) {
          break;
        }
        
        col = i == 0 ? attenuation : col * attenuation;
        r = scattered;

        
        vec3 ldir = getConeSample(L_POS-h.p, 1e-5);
        float llight = dot(h.normal, ldir);
        ray r2 = new_ray(h.p, ldir);
        hit h2;
        ray s2;
        vec3 c2;
        if(llight > 0.0 && !material_scatter(r, h2, c2, s2)) {
            col = col * llight * 1e-5;
        }
      } else {
        return emitted;
      }
    } else {
      return emitted;
    }
  }

  return emitted;
}

ray get_cam_ray(float vfov, float aspect, vec2 uv, float focus_dist, float aperture) {
  float theta = vfov * PI / 180.0;
  float half_height = tan(theta / 2.0);
  float half_width = aspect * half_height;
  vec3 w = normalize(in_eye - in_target);
  vec3 u = normalize(cross(in_up, w));
  vec3 v = cross(w, u);

  vec3 horizontal = focus_dist * half_width * u;
  vec3 vertical = focus_dist * half_height * v;
  vec3 lower_left_corner = in_eye - horizontal/2.0 - vertical/2.0 - focus_dist*w;

  vec2 jitter = hash2(g_seed) / resolution.xy;
  float lens_radius = aperture / 2.0;

  vec3 rd = lens_radius * random_in_unit_disk();
  vec3 offset = u * rd.x + v * rd.y;

  return new_ray(in_eye + offset, lower_left_corner + (uv.x + jitter.x) * horizontal + (uv.y + jitter.y) * vertical - in_eye - offset);
}

void main() {
  vec2 uv = gl_FragCoord.xy / resolution.xy;
  float aspect = resolution.x / resolution.y;

  // Init the seed. Make it different for each pixel + frame.
  g_seed = float(base_hash(floatBitsToUint(gl_FragCoord.xy))) / float(0xffffffffU) + time;

  // dist to target
  ray r = get_cam_ray(90.0, aspect, in_focus_pos, 1.0, 0.0);
  hit h;
  hit_scene(r, T_MIN, T_MAX, h);
  float f_dist = sqrt(dot(in_eye-h.p, in_eye-h.p));
  vec3 col = vec3(0.0);
  for (int i = 0; i < SAMPLING; ++i) {
    ray r = get_cam_ray(90.0, aspect, uv, f_dist, in_aperture);
    col += color(r);
  }

  col /= float(SAMPLING);

  FragColor = vec4(col, 1.0);
}