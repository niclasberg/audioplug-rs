// Blur functions adapted from https://madebyevan.com/shaders/fast-rounded-rectangle-shadows/

// Signed distance to a rounded rectangle centered at (0, 0)
float sdRoundedRect(float2 p, float2 halfSize, float radius) {
    float2 q = abs(p) - halfSize + radius;
    return length(max(q, 0.0)) - radius;
}

// Returns 1.0 if inside the rounded rect, 0.0 if outside
float isPointInRect(float2 origin, float2 size, float2 position) {
    float2 rel = position - origin;
    float2 inside = step(0.0, rel) * step(rel, size);
    return inside.x * inside.y;
}

// Returns 1.0 if inside the rounded rect, 0.0 if outside
float isPointInRoundedRect(float2 origin, float2 size, float radius, float2 position) {
    float2 p = position - (origin + size * 0.5);
    float dist = sdRoundedRect(p, size * 0.5, radius);
    // Anti-alias
    return saturate(0.5 - 100.0 * dist);
}

float4 erf4(float4 x) {
  float4 s = sign(x), a = abs(x);
  x = 1.0 + (0.278393 + (0.230389 + 0.078108 * (a * a)) * a) * a;
  x *= x;
  return s - s / (x * x);
}

float gaussian(float x, float sigma) {
  const float pi = 3.141592653589793;
  return exp(-(x * x) / (2.0 * sigma * sigma)) / (sqrt(2.0 * pi) * sigma);
}

// Approximation of the error function (i.e. integral of gaussian)
float2 erf2(float2 x) {
  float2 s = sign(x), a = abs(x);
  x = 1.0 + (0.278393 + (0.230389 + 0.078108 * (a * a)) * a) * a;
  x *= x;
  return s - s / (x * x);
}