#version 410
out vec4 FragColor;

in vec2 TexCoords;

uniform float time;
uniform vec2 resolution;
uniform float frame_nb;

uniform sampler2D pathbuffer;


// Stole from FMS_Cat : https://www.shadertoy.com/view/ss23DD

const vec4 LIFT = vec4( 0.02, 0.01, 0.01, 0.01 ) - vec4(0.02);
const vec4 GAMMA = vec4( 0.03, -0.01, 0.02, 0.00 );
const vec4 GAIN = vec4( 1.35, 1.21, 1.12, 1.24 );
const vec3 LUMA = vec3( 0.2126, 0.7152, 0.0722 );

vec3 liftGammaGain( vec3 rgb, vec4 lift, vec4 gamma, vec4 gain ) {
  vec4 liftt = 1.0 - pow( 1.0 - lift, log2( gain + 1.0 ) );

  vec4 gammat = gamma.rgba - vec4( 0.0, 0.0, 0.0, dot( LUMA, gamma.rgb ) );
  vec4 gammatTemp = 1.0 + 4.0 * abs( gammat );
  gammat = mix( gammatTemp, 1.0 / gammatTemp, step( 0.0, gammat ) );

  vec3 col = rgb;
  float luma = dot( LUMA, col );

  col = pow( col, gammat.rgb );
  col *= pow( gain.rgb, gammat.rgb );
  col = max( mix( 2.0 * liftt.rgb, vec3( 1.0 ), col ), 0.0 );

  luma = pow( luma, gammat.a );
  luma *= pow( gain.a, gammat.a );
  luma = max( mix( 2.0 * liftt.a, 1.0, luma ), 0.0 );

  col += luma - dot( LUMA, col );

  return col;
}

// Brightness - contrast - saturation
// From https://www.shadertoy.com/view/XdcXzn

mat4 brightnessMatrix( float brightness )
{
  return mat4( 1, 0, 0, 0,
                0, 1, 0, 0,
                0, 0, 1, 0,
                brightness, brightness, brightness, 1 );
}

mat4 contrastMatrix( float contrast )
{
	float t = ( 1.0 - contrast ) / 2.0;
   return mat4( contrast, 0, 0, 0,
                0, contrast, 0, 0,
                0, 0, contrast, 0,
                t, t, t, 1 );

}

mat4 saturationMatrix( float saturation )
{
    vec3 luminance = vec3( 0.3086, 0.6094, 0.0820 );
    float oneMinusSat = 1.0 - saturation;
    vec3 red = vec3( luminance.x * oneMinusSat );
    red+= vec3( saturation, 0, 0 );
    vec3 green = vec3( luminance.y * oneMinusSat );
    green += vec3( 0, saturation, 0 );
    vec3 blue = vec3( luminance.z * oneMinusSat );
    blue += vec3( 0, 0, saturation );
    return mat4( red,     0,
                 green,   0,
                 blue,    0,
                 0, 0, 0, 1 );
}



vec3 brightnessContrast(vec3 value, float brightness, float contrast)
{
    return (value - 0.5) * contrast + 0.5 + brightness;
}

vec4 brightContSat(vec4 c) {
  const float brightness = 0.0;
  const float contrast = 1.00;
  const float saturation = 1.05;
  return brightnessMatrix(brightness) * contrastMatrix(contrast) * saturationMatrix(saturation) * c;
}

void main() {
  vec2 uv = gl_FragCoord.xy / resolution.xy;
  vec4 col = vec4(texture(pathbuffer, uv).rgb / texture(pathbuffer, uv).a, 1.0);

  // color grading
  col.rgb = liftGammaGain(col.rgb, LIFT, GAMMA, GAIN);
  col.rgba = contrastMatrix(1.1) * col;

  // Tone mapping by Jim Hejl and Richard Burgess-Dawson
  vec3 x = max(vec3(0.0), col.rgb - vec3(0.004));
  vec3 retColor = (x*(6.2*x+.5))/(x*(6.2*x+1.7)+0.06);

  FragColor = col;
}