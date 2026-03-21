#version 330

in vec2 fragTexCoord;
in vec4 fragColor;

out vec4 finalColor;

uniform sampler2D texture0;
uniform vec4 colDiffuse;

void main() {
    vec2 uv = fragTexCoord;

    vec2 center = uv - 0.5;

    // Chromatic aberration
    float aberration = 0.001;
    float r = texture(texture0, uv + vec2(aberration, 0.0)).r;
    float g = texture(texture0, uv).g;
    float b = texture(texture0, uv - vec2(aberration, 0.0)).b;
    vec3 color = vec3(r, g, b);

    // Scanlines
    float scanline = sin(uv.y * 540.0 * 3.14159) * 0.5 + 0.5;
    color *= 0.93 + 0.07 * scanline;

    // Brightness compensation
    color *= 1.05;

    finalColor = vec4(color, 1.0);
}
