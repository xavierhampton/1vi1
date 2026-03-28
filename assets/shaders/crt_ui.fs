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
    color *= 0.88 + 0.12 * scan;

    // No vignette on UI — keeps text and icons fully readable

    finalColor = vec4(color, alpha);
}
