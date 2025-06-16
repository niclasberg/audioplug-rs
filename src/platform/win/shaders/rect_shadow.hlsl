#define D2D_INPUT_COUNT 0
#define D2D_REQUIRES_SCENE_POSITION
#include <d2d1effecthelpers.hlsli>
#include "common.hlsli"

cbuffer Constants : register(b0)
{
    float4 shadow_color;
    float2 size;
    float2 shadow_offset;
    float shadow_radius;
    float3 padding;
};

// Return the mask for a blurred rectangle 
float blurredRect(float2 lower, float2 upper, float2 position, float sigma) {
  float4 query = float4(lower - position, upper - position); 
  float4 integral = 0.5 + 0.5 * erf4(query * (sqrt(0.5) / sigma));
  return (integral.z - integral.x) * (integral.w - integral.y);
}

D2D_PS_ENTRY(RectShadowMain) {
    float2 pos = D2DGetScenePosition().xy;
    float mask = blurredRect(shadow_offset, shadow_offset + size, pos, shadow_radius / 3.0);
    mask *= 1.0 - isPointInRect(float2(0.0, 0.0), size, pos);
    return mask * shadow_color;
}
