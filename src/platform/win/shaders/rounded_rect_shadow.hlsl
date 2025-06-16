#define D2D_INPUT_COUNT 0
#define D2D_REQUIRES_SCENE_POSITION
#include <d2d1effecthelpers.hlsli>
#include "common.hlsli"

cbuffer Constants : register(b0)
{
    float4 shadow_color;
    float2 size;
    float2 shadow_offset;
    float corner_radius;
    float shadow_radius;
    float2 padding;
};

float roundedBoxShadowX(float x, float y, float sigma, float corner, float2 halfSize) {
  float delta = min(halfSize.y - corner - abs(y), 0.0);
  float curved = halfSize.x - corner + sqrt(max(0.0, corner * corner - delta * delta));
  float2 integral = 0.5 + 0.5 * erf2((x + float2(-curved, curved)) * (sqrt(0.5) / sigma));
  return integral.y - integral.x;
}

// Return the mask for the shadow of a box from lower to upper
float roundedBoxShadow(float2 lower, float2 upper, float2 position, float sigma, float corner) {
  // Center everything to make the math easier
  float2 center = (lower + upper) * 0.5;
  float2 halfSize = (upper - lower) * 0.5;
  position -= center;

  // The signal is only non-zero in a limited range, so don't waste samples
  float low = position.y - halfSize.y;
  float high = position.y + halfSize.y;
  float start = clamp(-3.0 * sigma, low, high);
  float end = clamp(3.0 * sigma, low, high);

  // Accumulate samples (we can get away with surprisingly few samples)
  float step = (end - start) / 4.0;
  float y = start + step * 0.5;
  float value = 0.0;
  for (int i = 0; i < 4; i++) {
    value += roundedBoxShadowX(position.x, position.y - y, sigma, corner, halfSize) * gaussian(y, sigma) * step;
    y += step;
  }

  return value;
}

D2D_PS_ENTRY(RoundedShadowMain) {
    float2 pos = D2DGetScenePosition().xy;
    float mask = roundedBoxShadow(shadow_offset, shadow_offset + size, pos, shadow_radius / 3.0, corner_radius);
    mask *= 1.0 - isPointInRoundedRect(float2(0.0, 0.0), size, corner_radius, pos);
    return mask * shadow_color;
}
