#version 330

in vec2 fragTexCoord;
in vec4 fragColor;

out vec4 finalColor;

uniform sampler2D texture0;
uniform vec4 colDiffuse;

void main() {
    vec2 uv = fragTexCoord;

    vec3 color = texture(texture0, uv).rgb;

    // Scanlines
    float scanline = sin(uv.y * 540.0 * 3.14159) * 0.5 + 0.5;
    color *= 0.93 + 0.07 * scanline;

    // Brightness compensation
    color *= 1.05;

    float alpha = texture(texture0, uv).a;
    finalColor = vec4(color, alpha);
}
