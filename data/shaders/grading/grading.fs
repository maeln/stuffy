#version 430
out vec4 FragColor;

in vec2 TexCoords;

uniform float time;
uniform vec2 resolution;
uniform float frame_nb;

layout(binding = 0) uniform sampler2D backbuffer;

void main() {
  vec2 uv = gl_FragCoord.xy / resolution.xy;
  vec3 col = texture(backbuffer, uv).rgb / texture(backbuffer, uv).a;

  // color grading
  col = pow(col, vec3(0.7, 0.8, 0.9));

  // exposure
  col *= exp2(1.1);

  // gamma
  col = pow(col, vec3(1.0/2.2));
  FragColor = vec4(col, 1.0);
}