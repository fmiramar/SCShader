void mainImage(out vec4 fragColor, in vec2 fragCoord) {
    vec2 uv = fragCoord / iResolution.xy;
    vec2 mouse = iMouse.xy / max(iResolution.xy, vec2(1.0));
    float dayPhase = fract(iDate.w / 86400.0);
    float distanceToMouse = length(uv - mouse);
    fragColor = vec4(dayPhase, 0.3 + 0.7 * exp(-8.0 * distanceToMouse), uv.y, 1.0);
}
