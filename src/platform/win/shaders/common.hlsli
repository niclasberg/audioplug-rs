// Blur functions adapted from https://madebyevan.com/shaders/fast-rounded-rectangle-shadows/

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

float4 erf4(float4 x) {
  float4 s = sign(x), a = abs(x);
  x = 1.0 + (0.278393 + (0.230389 + 0.078108 * (a * a)) * a) * a;
  x *= x;
  return s - s / (x * x);
}

// Return the mask for a blurred rectangle 
float blurredRect(float2 origin, float2 size, float2 position, float sigma) {
  float4 query = float4(position - origin, origin + size - position);
  float4 integral = 0.5 + 0.5 * erf4(query * (sqrt(0.5) / sigma));
  return (integral.z - integral.x) * (integral.w - integral.y);
}