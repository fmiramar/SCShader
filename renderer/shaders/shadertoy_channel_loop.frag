void mainImage(out vec4 fragColor, in vec2 fragCoord) {
    vec2 uv = fragCoord / iResolution.xy;
    vec3 color = texture(iChannel0, uv).rgb;
    for(int index = 0; index < 3; index++) {
        color += 0.04 * sin(vec3(1.0, 1.7, 2.3) * (iTime + float(index)));
    }
    float edge = fwidth(uv.x);
    fragColor = vec4(color + vec3(edge), 1.0);
}
