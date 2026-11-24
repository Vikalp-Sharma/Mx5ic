#version 330 core
out vec4 FragColor;

uniform vec2 u_resolution;
uniform float u_time;
uniform vec2 u_pointer;     // Normalized cursor coordinates [0, 1]
uniform float u_audio_rms;   // Audio intensity [0, 1]
uniform float u_bypass;      // 0.0 = Active AI Enhanced, 1.0 = Bypass

// -----------------------------------------------------------------------------
// SDF Primitives & Operations
// -----------------------------------------------------------------------------
float sdRoundedBox(vec2 p, vec2 b, vec4 r) {
    r.xy = (p.x > 0.0) ? r.xy : r.zw;
    r.x  = (p.y > 0.0) ? r.x  : r.y;
    vec2 q = abs(p) - b + r.x;
    return min(max(q.x, q.y), 0.0) + length(max(q, 0.0)) - r.x;
}

float sdCircle(vec2 p, float r) {
    return length(p) - r;
}

float sdSegment(vec2 p, vec2 a, vec2 b) {
    vec2 pa = p - a, ba = b - a;
    float h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h);
}

float smin(float a, float b, float k) {
    float h = clamp(0.5 + 0.5 * (b - a) / k, 0.0, 1.0);
    return mix(b, a, h) - k * h * (1.0 - h);
}

// -----------------------------------------------------------------------------
// Living Background: Deep Obsidian Aurora Gradient
// -----------------------------------------------------------------------------
vec3 renderBackground(vec2 uv, float t) {
    vec2 p = uv * 2.0 - 1.0;
    p.x *= u_resolution.x / u_resolution.y;

    float wave1 = sin(p.x * 2.2 + t * 0.7 + u_audio_rms * 1.2) * 0.35;
    float wave2 = cos(p.y * 2.8 - t * 0.5) * 0.25;
    float d1 = length(p + vec2(wave1, wave2) - vec2(0.35, -0.2));
    float d2 = length(p - vec2(wave2, wave1) - vec2(-0.45, 0.3));

    vec3 colDeep   = vec3(0.03, 0.04, 0.07);      // Deep Midnight Obsidian
    vec3 colCyan   = vec3(0.06, 0.65, 0.85);      // Electric Cyan
    vec3 colViolet = vec3(0.55, 0.18, 0.90);      // Amethyst Violet
    vec3 colAmber  = vec3(0.98, 0.48, 0.12);      // Sunset Amber Glow

    if (u_bypass > 0.5) {
        colCyan   = vec3(0.25, 0.35, 0.45);
        colViolet = vec3(0.40, 0.25, 0.35);
        colAmber  = vec3(0.60, 0.40, 0.20);
    }

    vec3 bg = mix(colDeep, colViolet, smoothstep(1.5, 0.1, d1) * 0.7);
    bg = mix(bg, colCyan, smoothstep(1.3, 0.05, d2) * 0.65);
    bg += colAmber * 0.15 * (sin(p.x * 3.5 + t * 0.9) * 0.5 + 0.5) * (u_audio_rms + 0.25);

    // Subtle dark vignette
    float vig = smoothstep(1.4, 0.4, length(p * 0.75));
    return bg * (0.45 + 0.55 * vig);
}

// -----------------------------------------------------------------------------
// Composite Scene SDF: Concentric Liquid Glass Surfaces
// -----------------------------------------------------------------------------
float sceneSDF(vec2 p) {
    // 1. Top Header Capsule Bar
    float dHeader = sdRoundedBox(p - vec2(0.0, 0.72), vec2(0.88, 0.09), vec4(0.08));

    // 2. Central Enhance Dial & Glass Dome
    float orbR = 0.30 + 0.02 * u_audio_rms;
    float dCenterDial = sdCircle(p - vec2(0.0, 0.15), orbR);

    // 3. 3D Spatial Orbit Stage Panel (Below Center Dial)
    float dSpatialStage = sdRoundedBox(p - vec2(0.0, -0.32), vec2(0.42, 0.18), vec4(0.09));

    // 4. Left Panel: Input Spectrum Card
    float dLeftCard = sdRoundedBox(p - vec2(-0.95, -0.05), vec2(0.32, 0.42), vec4(0.08));

    // 5. Right Panel: Enhanced Output Spectrum Card
    float dRightCard = sdRoundedBox(p - vec2(0.95, -0.05), vec2(0.32, 0.42), vec4(0.08));

    // 6. Bottom Control Pill Dock
    float dBottomDock = sdRoundedBox(p - vec2(0.0, -0.74), vec2(0.85, 0.08), vec4(0.07));

    // Smooth organic fluid union
    float k = 0.06;
    float d = smin(dHeader, dCenterDial, k);
    d = smin(d, dSpatialStage, k * 1.5);
    d = smin(d, dLeftCard, k);
    d = smin(d, dRightCard, k);
    d = smin(d, dBottomDock, k);

    return d;
}

// Compute 2D surface normal from SDF gradient
vec2 calcNormal(vec2 p) {
    float eps = 0.002;
    return normalize(vec2(
        sceneSDF(p + vec2(eps, 0.0)) - sceneSDF(p - vec2(eps, 0.0)),
        sceneSDF(p + vec2(0.0, eps)) - sceneSDF(p - vec2(0.0, eps))
    ));
}

// -----------------------------------------------------------------------------
// UI Overlays: 3D Vocal Orbit Stage & Spectrum Meters
// -----------------------------------------------------------------------------
vec3 renderUIOverlay(vec2 p, vec2 uv, float t) {
    vec3 overlay = vec3(0.0);

    // 1. 3D Spatial Vocal Orbit Stage (p around vec2(0.0, -0.32))
    vec2 stageCenter = vec2(0.0, -0.32);
    vec2 sp = p - stageCenter;
    if (abs(sp.x) < 0.40 && abs(sp.y) < 0.16) {
        // Head in center
        float dHead = sdCircle(sp, 0.045);
        overlay += vec3(0.2, 0.35, 0.5) * smoothstep(0.004, 0.0, dHead);

        // Left earbud & Right earbud markers
        float dLeftEar  = sdCircle(sp - vec2(-0.24, 0.0), 0.022);
        float dRightEar = sdCircle(sp - vec2( 0.24, 0.0), 0.022);
        overlay += vec3(0.0, 0.8, 1.0) * smoothstep(0.003, 0.0, dLeftEar);
        overlay += vec3(1.0, 0.4, 0.8) * smoothstep(0.003, 0.0, dRightEar);

        // Orbit path: dotted/faint ellipse
        float orbitR = length(vec2(sp.x / 1.6, sp.y / 0.65));
        float dOrbitPath = abs(orbitR - 0.15);
        overlay += vec3(0.3, 0.5, 0.7) * smoothstep(0.005, 0.0, dOrbitPath) * 0.4;

        // Dynamic 3D Orbiting Vocal Dot (Earbud Ping-Pong: sweeping L -> R -> L)
        float orbitAngle = t * 2.2;
        vec2 vocalPos = vec2(sin(orbitAngle) * 0.24, cos(orbitAngle) * 0.07);
        float dVocal = length(sp - vocalPos);

        // Vibrant glowing vocal core
        float vocalGlow = exp(-dVocal * 45.0) * 1.5;
        vec3 vocalColor = mix(vec3(0.0, 0.9, 1.0), vec3(1.0, 0.3, 0.9), sin(orbitAngle) * 0.5 + 0.5);
        overlay += vocalColor * vocalGlow;

        // Motion fade trailing particles (simulating sound fading into distance)
        for (int i = 1; i <= 4; i++) {
            float trailAngle = orbitAngle - float(i) * 0.18;
            vec2 trailPos = vec2(sin(trailAngle) * 0.24, cos(trailAngle) * 0.07);
            float dTrail = length(sp - trailPos);
            overlay += vocalColor * exp(-dTrail * 60.0) * (0.6 / float(i));
        }
    }

    // 2. Left Panel: Raw Input Spectrum Bars (8 bands)
    vec2 lp = p - vec2(-0.95, -0.05);
    if (abs(lp.x) < 0.26 && abs(lp.y) < 0.34) {
        float barWidth = 0.024;
        float spacing = 0.034;
        for (int i = 0; i < 8; i++) {
            float bx = -0.12 + float(i) * spacing;
            // Unenhanced dull roll-off at high frequencies
            float rawHeight = (0.28 - float(i) * 0.028) * (0.6 + 0.4 * sin(t * 4.0 + float(i) * 1.5));
            float dBar = sdRoundedBox(lp - vec2(bx, -0.22 + rawHeight * 0.5), vec2(barWidth * 0.4, rawHeight * 0.5), vec4(0.005));
            if (dBar < 0.0) {
                // Dull amber/gray color for raw input
                overlay += vec3(0.55, 0.45, 0.35) * 0.85;
            }
        }
    }

    // 3. Right Panel: Enhanced Output Spectrum Bars (8 bands with restored HF air)
    vec2 rp = p - vec2(0.95, -0.05);
    if (abs(rp.x) < 0.26 && abs(rp.y) < 0.34) {
        float barWidth = 0.024;
        float spacing = 0.034;
        for (int i = 0; i < 8; i++) {
            float bx = -0.12 + float(i) * spacing;
            // Enhanced full bandwidth extension (vibrant highs + punchy virtual bass)
            float enhHeight = (0.20 + 0.08 * sin(float(i) * 0.8)) * (0.7 + 0.5 * sin(t * 4.5 + float(i) * 1.8 + u_audio_rms * 2.0));
            enhHeight = clamp(enhHeight, 0.04, 0.30);
            float dBar = sdRoundedBox(rp - vec2(bx, -0.22 + enhHeight * 0.5), vec2(barWidth * 0.4, enhHeight * 0.5), vec4(0.005));
            if (dBar < 0.0) {
                // Vibrant electric cyan to magenta gradient for AI restored sound
                vec3 barCol = mix(vec3(0.0, 0.85, 0.95), vec3(0.95, 0.25, 0.85), float(i) / 7.0);
                if (u_bypass > 0.5) barCol = vec3(0.4, 0.4, 0.4);
                overlay += barCol * 1.1;
            }
        }
    }

    // 4. Central Dial Status Rings & Radial Markings
    vec2 cp = p - vec2(0.0, 0.15);
    float dCenterDist = length(cp);
    if (dCenterDist > 0.18 && dCenterDist < 0.28) {
        float ringDash = sin(atan(cp.y, cp.x) * 24.0);
        if (ringDash > 0.3) {
            vec3 ringCol = (u_bypass > 0.5) ? vec3(0.6, 0.3, 0.2) : vec3(0.0, 0.9, 0.8);
            overlay += ringCol * 0.5 * smoothstep(0.28, 0.25, dCenterDist);
        }
    }

    // Center Core Indicator
    if (dCenterDist < 0.08) {
        vec3 coreCol = (u_bypass > 0.5) ? vec3(0.85, 0.25, 0.20) : vec3(0.05, 0.95, 0.75);
        overlay += coreCol * (0.8 + 0.4 * u_audio_rms);
    }

    return overlay;
}

// -----------------------------------------------------------------------------
// Main Shader Entry
// -----------------------------------------------------------------------------
void main() {
    vec2 uv = gl_FragCoord.xy / u_resolution;
    vec2 p  = (gl_FragCoord.xy * 2.0 - u_resolution) / min(u_resolution.x, u_resolution.y);
    float t = u_time * 0.8;

    float dist = sceneSDF(p);

    if (dist > 0.0) {
        // Outside glass surfaces: Living aurora background + soft ambient edge glow
        vec3 bg = renderBackground(uv, t);
        float glow = exp(-dist * 14.0) * 0.30;
        vec3 glowCol = (u_bypass > 0.5) ? vec3(0.6, 0.4, 0.3) : vec3(0.3, 0.7, 1.0);
        FragColor = vec4(bg + glowCol * glow, 1.0);
        return;
    }

    // Inside liquid glass: Calculate refractive optics & dispersion
    vec2 normal2D = calcNormal(p);
    vec3 normal = normalize(vec3(normal2D, 0.65));
    vec3 viewDir = vec3(0.0, 0.0, 1.0);

    // Light source following cursor / pointer
    vec2 ptrPos = (u_pointer * 2.0 - 1.0);
    ptrPos.x *= u_resolution.x / u_resolution.y;
    vec3 lightDir = normalize(vec3(ptrPos - p, 0.85));

    // 1. Chromatic Dispersion Refraction (RGB ray splitting)
    float etaR = 0.73;
    float etaG = 0.70;
    float etaB = 0.67;

    vec2 refrR = refract(-viewDir, normal, etaR).xy * 0.10;
    vec2 refrG = refract(-viewDir, normal, etaG).xy * 0.10;
    vec2 refrB = refract(-viewDir, normal, etaB).xy * 0.10;

    vec3 refrColor;
    refrColor.r = renderBackground(uv + refrR, t).r;
    refrColor.g = renderBackground(uv + refrG, t).g;
    refrColor.b = renderBackground(uv + refrB, t).b;

    // 2. Crystal Tint & Translucency
    vec3 glassTint = (u_bypass > 0.5) ? vec3(0.85, 0.80, 0.80) : vec3(0.88, 0.94, 1.0);
    refrColor *= glassTint;

    // 3. Specular Fresnel Rim
    float fresnel = pow(1.0 - max(dot(viewDir, normal), 0.0), 3.2);
    vec3 specularRim = vec3(1.0) * fresnel * 0.90;

    // 4. Pointer Specular Glint (Blinn-Phong)
    vec3 halfVec = normalize(lightDir + viewDir);
    float spec = pow(max(dot(normal, halfVec), 0.0), 36.0);
    vec3 pointerSpecular = vec3(1.0, 0.96, 0.92) * spec * 1.35;

    // 5. Soft Bevel / Inner Shadow
    float edgeDist = abs(dist);
    float innerBevel = smoothstep(0.0, 0.035, edgeDist);

    // Combine base glass shading
    vec3 glassColor = refrColor * (0.82 + 0.18 * innerBevel) + specularRim + pointerSpecular;

    // 6. Draw UI Elements & 3D Spatial Stage Overlays
    vec3 uiOverlay = renderUIOverlay(p, uv, t);
    glassColor += uiOverlay;

    // Smooth outer border anti-aliasing
    float alpha = smoothstep(0.002, -0.002, dist);
    vec3 bgSample = renderBackground(uv, t);
    vec3 finalColor = mix(bgSample, glassColor, alpha);

    FragColor = vec4(finalColor, 1.0);
}
