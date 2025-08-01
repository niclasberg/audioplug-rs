
// Fresnel-Schlick
float3 FresnelSchlick(float cosTheta, float3 F0)
{
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}

// GGX NDF
float D_GGX(float NdotH, float roughness)
{
    float a = roughness * roughness;
    float a2 = a * a;
    float d = (NdotH * NdotH) * (a2 - 1.0) + 1.0;
    return a2 / (PI * d * d + 1e-5);
}

// Geometry function
float G_Smith(float NdotV, float NdotL, float roughness)
{
    float r = (roughness + 1.0);
    float k = (r * r) / 8.0;

    float G1V = NdotV / (NdotV * (1.0 - k) + k);
    float G1L = NdotL / (NdotL * (1.0 - k) + k);
    return G1V * G1L;
}

// Disney lighting
float3 DisneyLighting(float3 N, float3 L, float3 V, float3 baseColor, float metallic, float roughness)
{
    float3 H = normalize(V + L);
    float NdotL = clamp01(dot(N, L));
    float NdotV = clamp01(dot(N, V));
    float NdotH = clamp01(dot(N, H));
    float LdotH = clamp01(dot(L, H));

    float3 F0 = lerp(float3(0.04, 0.04, 0.04), baseColor, metallic);
    float3 F = FresnelSchlick(LdotH, F0);
    float D = D_GGX(NdotH, roughness);
    float G = G_Smith(NdotV, NdotL, roughness);

    float3 spec = (D * G * F) / (4.0 * NdotL * NdotV + 1e-5);
    float energyFactor = lerp(1.0, 1.0 / 1.51, roughness);
    float3 diffuse = (1.0 - F) * baseColor / PI;

    return (diffuse + spec) * NdotL * energyFactor;
}