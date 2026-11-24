#version 330 core
out vec4 FragColor;

uniform vec2 u_resolution;
uniform float u_time;
uniform vec2 u_pointer; // Normalized pointer coords [0, 1]
uniform float u_audio_rms; // Audio reactivity level [0, 1]

// Signed Distance Function (SDF) for a rounded box
float sdRoundedBox(vec2 p, vec2 b, vec4 r) {
    r.xy = (p.x > 0.0) ? r.xy : r.zw;
    r.x  = (p.y > 0.0) ? r.x  : r.y;
    vec2 q = abs(p) - b + r.x;
    return min(max(q.x, q.y), 0.0) + length(max(q, 0.0)) - r.x;
}

// SDF for a circle
float sdCircle(vec2 p, float r) {
    return length(p) - r;
}

// Polynomial smooth minimum for fluid/organic merging
float smin(float a, float b, float k) {
    float h = clamp(0.5 + 0.5 * (b - a) / k, 0.0, 1.0);
    return mix(b, a, h) - k * h * (1.0 - h);
}

// Living background: generative aurora/gradient mesh
vec3 renderBackground(vec2 uv, float t) {
    vec2 p = uv * 2.0 - 1.0;
    p.x *= u_resolution.x / u_resolution.y;

    float wave1 = sin(p.x * 2.5 + t * 0.8 + u_audio_rms * 1.5) * 0.35;
    float wave2 = cos(p.y * 3.0 - t * 0.6) * 0.25;
    float d1 = length(p + vec2(wave1, wave2) - vec2(0.3, -0.2));
    float d2 = length(p - vec2(wave2, wave1) - vec2(-0.4, 0.3));

    vec3 colDeep = vec3(0.04, 0.05, 0.09);       // Deep Obsidian
    vec3 colCyan = vec3(0.08, 0.55, 0.75);       // Electric Cyan
    vec3 colViolet = vec3(0.48, 0.15, 0.82);     // Radiant Violet
    vec3 colAmber = vec3(0.95, 0.45, 0.15);      // Warm Amber glow

    vec3 bg = mix(colDeep, colViolet, smoothstep(1.4, 0.1, d1));
    bg = mix(bg, colCyan, smoothstep(1.2, 0.05, d2) * 0.8);
    bg += colAmber * 0.18 * (sin(p.x * 4.0 + t) * 0.5 + 0.5) * (u_audio_rms + 0.2);

    return bg;
}

// Scene SDF containing concentric liquid glass elements
float sceneSDF(vec2 p) {
    // 1. Central "Enhance" Glass Orb (pulsing subtly with audio)
    float orbRadius = 0.32 + 0.03 * u_audio_rms;
    float dOrb = sdCircle(p - vec2(0.0, 0.0), orbRadius);

    // 2. Mode Capsule (Top)
    float dCapsuleTop = sdRoundedBox(p - vec2(0.0, 0.52), vec2(0.38, 0.08), vec4(0.08));

    // 3. Floating Left/Right Controls
    float dLeft = sdRoundedBox(p - vec2(-0.55, -0.15), vec2(0.18, 0.18), vec4(0.09));
    float dRight = sdRoundedBox(p - vec2(0.55, -0.15), vec2(0.18, 0.18), vec4(0.09));

    // 4. Smooth fluid organic union between elements
    float blendFactor = 0.12;
    float d = smin(dOrb, dCapsuleTop, blendFactor);
    d = smin(d, dLeft, blendFactor);
    d = smin(d, dRight, blendFactor);

    return d;
}

// Compute normal from 2D SDF gradient
vec2 calcNormal(vec2 p) {
    float eps = 0.002;
    return normalize(vec2(
        sceneSDF(p + vec2(eps, 0.0)) - sceneSDF(p - vec2(eps, 0.0)),
        sceneSDF(p + vec2(0.0, eps)) - sceneSDF(p - vec2(0.0, eps))
    ));
}

void main() {
    vec2 uv = gl_FragCoord.xy / u_resolution;
    vec2 p = (gl_FragCoord.xy * 2.0 - u_resolution) / min(u_resolution.x, u_resolution.y);
    float t = u_time * 0.7;

    // Evaluate glass SDF
    float dist = sceneSDF(p);

    if (dist > 0.0) {
        // Outside glass: render pure living background with subtle ambient glow near edges
        vec3 bg = renderBackground(uv, t);
        float glow = exp(-dist * 12.0) * 0.25;
        vec3 finalColor = bg + vec3(0.4, 0.7, 1.0) * glow;
        FragColor = vec4(finalColor, 1.0);
        return;
    }

    // Inside glass surface: calculate liquid optics
    vec2 normal2D = calcNormal(p);
    vec3 normal = normalize(vec3(normal2D, 0.65)); // Pseudo 3D dome normal
    vec3 viewDir = vec3(0.0, 0.0, 1.0);

    // Light source reacting to pointer / cursor
    vec2 ptrPos = (u_pointer * 2.0 - 1.0);
    ptrPos.x *= u_resolution.x / u_resolution.y;
    vec3 lightDir = normalize(vec3(ptrPos - p, 0.8));

    // 1. Refraction & Chromatic Dispersion
    // Refract Red, Green, and Blue rays at slightly different indices of refraction (IOR)
    float etaR = 0.72; // Red index
    float etaG = 0.70; // Green index
    float etaB = 0.68; // Blue index (higher refraction)

    vec2 refrR = refract(-viewDir, normal, etaR).xy * 0.12;
    vec2 refrG = refract(-viewDir, normal, etaG).xy * 0.12;
    vec2 refrB = refract(-viewDir, normal, etaB).xy * 0.12;

    vec3 refrColor;
    refrColor.r = renderBackground(uv + refrR, t).r;
    refrColor.g = renderBackground(uv + refrG, t).g;
    refrColor.b = renderBackground(uv + refrB, t).b;

    // 2. Translucent Glass Tint (adaptive crystal clarity)
    vec3 glassTint = vec3(0.85, 0.92, 1.0);
    refrColor *= glassTint;

    // 3. Specular Rim Highlight (Fresnel reflection)
    float fresnel = pow(1.0 - max(dot(viewDir, normal), 0.0), 3.5);
    vec3 specularRim = vec3(1.0, 1.0, 1.0) * fresnel * 0.85;

    // 4. Dynamic Blinn-Phong Pointer Specular Glint
    vec3 halfVec = normalize(lightDir + viewDir);
    float spec = pow(max(dot(normal, halfVec), 0.0), 32.0);
    vec3 pointerSpecular = vec3(1.0, 0.95, 0.9) * spec * 1.2;

    // 5. Soft Inner Shadow / Border Occlusion
    float edgeDist = abs(dist);
    float innerBevel = smoothstep(0.0, 0.03, edgeDist);

    // Combine glass optics
    vec3 glassColor = refrColor * (0.8 + 0.2 * innerBevel) + specularRim + pointerSpecular;

    // Anti-aliased outer edge boundary
    float alpha = smoothstep(0.002, -0.002, dist);
    vec3 bgSample = renderBackground(uv, t);
    vec3 finalColor = mix(bgSample, glassColor, alpha);

    FragColor = vec4(finalColor, 1.0);
}
