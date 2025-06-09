#define D2D_INPUT_COUNT 0
#include <d2d1effecthelpers.hlsli>
#include "common.hlsli"

cbuffer Constants : register(b0)
{
    float2 size;
    float shadow_radius;
    float2 shadow_offset;
    float4 shadow_color;
};

D2D_PS_ENTRY(RectShadowMain) {
    
    return shadow_color;
}
