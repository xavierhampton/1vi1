#version 330

in vec2 fragTexCoord;
in vec4 fragColor;

out vec4 finalColor;

uniform sampler2D texture0;
uniform vec4 colDiffuse;

// CRT barrel distortion — contracts UVs inward so edges never leave [0,1]
vec2 barrel(vec2 uv) {
    vec2 c = uv - 0.5;
    float r2 = dot(c, c);
    return c / (1.0 + r2 * 0.2) + 0.5;
}

void main() {
    vec2 uv = barrel(fragTexCoord);

    // Radial chromatic aberration (fades near edges to prevent fringing)
    float edgeFade = smoothstep(0.0, 0.08, uv.x) * smoothstep(0.0, 0.08, 1.0 - uv.x)
                   * smoothstep(0.0, 0.08, uv.y) * smoothstep(0.0, 0.08, 1.0 - uv.y);
    vec2 dir = (uv - 0.5) * 0.0025 * edgeFade;
    float r = texture(texture0, uv + dir).r;
    float g = texture(texture0, uv).g;
    float b = texture(texture0, uv - dir).b;
    vec3 color = vec3(r, g, b);

    // Scanlines (thick, visible retro bands)
    float scan = smoothstep(0.35, 0.5, fract(uv.y * 300.0));
    color *= 0.78 + 0.22 * scan;

    // Vignette (dark corners)
    vec2 vig = uv - 0.5;
    float vigAmount = 1.0 - dot(vig, vig) * 1.8;
    color *= clamp(vigAmount, 0.0, 1.0);

    // Brightness compensation
    color *= 1.15;

    // Preserve source alpha (transparent areas stay transparent for overlay compositing)
    float alpha = texture(texture0, uv).a;
    finalColor = vec4(color, alpha);
}
