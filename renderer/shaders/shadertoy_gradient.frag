void mainImage(out vec4 fragColor, in vec2 fragCoord) {
    vec2 uv = fragCoord / iResolution.xy;
    float pulse = 0.5 + 0.5 * sin(iTime + float(iFrame) * 0.001);
    fragColor = vec4(uv, pulse, 1.0);
}
