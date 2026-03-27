#version 330

in vec2 fragTexCoord;
in vec4 fragColor;

out vec4 finalColor;

uniform sampler2D texture0;
uniform vec4 colDiffuse;

// No barrel distortion — UI stays at exact screen positions
void main() {
    vec2 uv = fragTexCoord;

    vec4 texel = texture(texture0, uv);
    vec3 color = texel.rgb;
    float alpha = texel.a;

    // Scanlines (light, matching other passes)
    float scan = smoothstep(0.35, 0.5, fract(uv.y * 300.0));
    color *= 0.85 + 0.15 * scan;

    // Vignette
    vec2 vig = uv - 0.5;
    float vigAmount = 1.0 - dot(vig, vig) * 1.2;
    color *= clamp(vigAmount, 0.0, 1.0);

    // Brightness compensation
    color *= 1.08;

    finalColor = vec4(color, alpha);
}
