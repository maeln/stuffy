#version 410
out vec4 FragColor;

in vec2 TexCoords;

uniform float time;
uniform vec2 resolution;
uniform float frame_nb;

uniform sampler2D backbuffer;

void main() {
  vec2 uv = gl_FragCoord.xy / resolution.xy;
  vec3 col = texture(backbuffer, uv).rgb / texture(backbuffer, uv).a;

  float ratio = uv.x / uv.y;
  float filmic_ratio = 1.85 / 1.0;

  // color grading
  col = pow(col, vec3(0.65, 0.85, 0.9));

  // exposure
  col *= exp2(0.8);

  // Tone mapping by Jim Hejl and Richard Burgess-Dawson
  vec3 x = max(vec3(0.0), col - vec3(0.004));
  vec3 retColor = (x*(6.2*x+.5))/(x*(6.2*x+1.7)+0.06);

  // gamma
  //col = pow(col, vec3(1.0/2.2));

//  if(uv.y < 0.1 || uv.y > 0.9) {
//    FragColor = vec4(vec3(0.0), 1.0);
//  } else {
    FragColor = vec4(col, 1.0);
//  }

  /*
  float3 rgb2yuv(float3 rgb) 
{
    return float3(    
    rgb.r * 0.299 + rgb.g * 0.587 + rgb.b * 0.114,
    rgb.r * -0.169 + rgb.g * -0.331 + rgb.b * 0.5 + 0.5,
    rgb.r * 0.5 + rgb.g * -0.419 + rgb.b * -0.081 + 0.5
    );
}
float3 YUV = rgb2yuv(c.rgb);
    ResultPoints[i.x].position = float3(
        (YUV.y-0.5) * radius,  
        (YUV.z-0.5) * radius,
        0 );
  */
}