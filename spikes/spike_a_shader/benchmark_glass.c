#define GL_GLEXT_PROTOTYPES 1
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <time.h>
#include <GL/gl.h>
#include <GL/glext.h>
#include <GLFW/glfw3.h>

#define WIDTH 1920
#define HEIGHT 1080
#define TOTAL_FRAMES 1000

static double get_time_sec(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (double)ts.tv_sec + (double)ts.tv_nsec * 1e-9;
}

static char* load_shader_source(const char* filepath) {
    FILE* f = fopen(filepath, "rb");
    if (!f) {
        fprintf(stderr, "Failed to open shader file: %s\n", filepath);
        return NULL;
    }
    fseek(f, 0, SEEK_END);
    long len = ftell(f);
    fseek(f, 0, SEEK_SET);
    char* src = malloc(len + 1);
    if (fread(src, 1, len, f) != (size_t)len) {
        fprintf(stderr, "Failed to read shader file: %s\n", filepath);
        free(src);
        fclose(f);
        return NULL;
    }
    src[len] = '\0';
    fclose(f);
    return src;
}

static void save_ppm(const char* filename, int width, int height, unsigned char* pixels) {
    FILE* f = fopen(filename, "wb");
    if (!f) return;
    fprintf(f, "P6\n%d %d\n255\n", width, height);
    for (int y = height - 1; y >= 0; y--) {
        for (int x = 0; x < width; x++) {
            int idx = (y * width + x) * 4;
            fputc(pixels[idx + 0], f);
            fputc(pixels[idx + 1], f);
            fputc(pixels[idx + 2], f);
        }
    }
    fclose(f);
}

static int cmp_double(const void* a, const void* b) {
    double da = *(const double*)a;
    double db = *(const double*)b;
    return (da > db) - (da < db);
}

int main(int argc, char** argv) {
    printf("====================================================\n");
    printf("   Spike a: Liquid Glass Fragment Shader Benchmark  \n");
    printf("   Target: 1920x1080 @ >=60 FPS on Linux / GPU     \n");
    printf("====================================================\n");

    if (!glfwInit()) {
        fprintf(stderr, "ERROR: Failed to initialize GLFW\n");
        return 1;
    }

    glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 3);
    glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 3);
    glfwWindowHint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);
    glfwWindowHint(GLFW_VISIBLE, GLFW_FALSE); // Offscreen / hidden window

    GLFWwindow* window = glfwCreateWindow(WIDTH, HEIGHT, "Liquid Glass Benchmark", NULL, NULL);
    if (!window) {
        fprintf(stderr, "ERROR: Failed to create GLFW window\n");
        glfwTerminate();
        return 1;
    }

    glfwMakeContextCurrent(window);
    glfwSwapInterval(0); // Disable VSync to measure uncapped raw shader framerate

    // Print GPU Renderer info
    printf("OpenGL Vendor   : %s\n", glGetString(GL_VENDOR));
    printf("OpenGL Renderer : %s\n", glGetString(GL_RENDERER));
    printf("OpenGL Version  : %s\n", glGetString(GL_VERSION));

    // Vertex Shader for full-screen quad
    const char* vert_src =
        "#version 330 core\n"
        "layout (location = 0) in vec2 aPos;\n"
        "void main() {\n"
        "    gl_Position = vec4(aPos, 0.0, 1.0);\n"
        "}\n";

    char* frag_src = load_shader_source("spikes/spike_a_shader/liquid_glass.frag");
    if (!frag_src) {
        glfwDestroyWindow(window);
        glfwTerminate();
        return 1;
    }

    GLuint vs = glCreateShader(GL_VERTEX_SHADER);
    glShaderSource(vs, 1, &vert_src, NULL);
    glCompileShader(vs);

    GLuint fs = glCreateShader(GL_FRAGMENT_SHADER);
    glShaderSource(fs, 1, (const GLchar**)&frag_src, NULL);
    glCompileShader(fs);

    GLint success;
    GLchar infoLog[1024];
    glGetShaderiv(fs, GL_COMPILE_STATUS, &success);
    if (!success) {
        glGetShaderInfoLog(fs, 1024, NULL, infoLog);
        fprintf(stderr, "ERROR: Fragment shader compilation failed:\n%s\n", infoLog);
        free(frag_src);
        return 1;
    }
    free(frag_src);

    GLuint program = glCreateProgram();
    glAttachShader(program, vs);
    glAttachShader(program, fs);
    glLinkProgram(program);

    glGetProgramiv(program, GL_LINK_STATUS, &success);
    if (!success) {
        glGetProgramInfoLog(program, 1024, NULL, infoLog);
        fprintf(stderr, "ERROR: Shader program linking failed:\n%s\n", infoLog);
        return 1;
    }

    glDeleteShader(vs);
    glDeleteShader(fs);

    // Quad geometry
    float quad_vertices[] = {
        -1.0f,  1.0f,
        -1.0f, -1.0f,
         1.0f, -1.0f,
        -1.0f,  1.0f,
         1.0f, -1.0f,
         1.0f,  1.0f
    };

    GLuint vao, vbo;
    glGenVertexArrays(1, &vao);
    glGenBuffers(1, &vbo);

    glBindVertexArray(vao);
    glBindBuffer(GL_ARRAY_BUFFER, vbo);
    glBufferData(GL_ARRAY_BUFFER, sizeof(quad_vertices), quad_vertices, GL_STATIC_DRAW);

    glVertexAttribPointer(0, 2, GL_FLOAT, GL_FALSE, 2 * sizeof(float), (void*)0);
    glEnableVertexAttribArray(0);

    GLint loc_res = glGetUniformLocation(program, "u_resolution");
    GLint loc_time = glGetUniformLocation(program, "u_time");
    GLint loc_ptr = glGetUniformLocation(program, "u_pointer");
    GLint loc_audio = glGetUniformLocation(program, "u_audio_rms");

    glUseProgram(program);
    glUniform2f(loc_res, (float)WIDTH, (float)HEIGHT);

    // Warm-up 20 frames
    for (int i = 0; i < 20; i++) {
        glUniform1f(loc_time, (float)i * 0.016f);
        glUniform2f(loc_ptr, 0.5f, 0.5f);
        glUniform1f(loc_audio, 0.4f);
        glDrawArrays(GL_TRIANGLES, 0, 6);
        glfwSwapBuffers(window);
        glFinish();
    }

    printf("[*] Starting benchmark: rendering %d frames @ 1080p...\n", TOTAL_FRAMES);

    double frame_times[TOTAL_FRAMES];
    unsigned char* screenshot_pixels = malloc(WIDTH * HEIGHT * 4);

    double total_start = get_time_sec();

    for (int i = 0; i < TOTAL_FRAMES; i++) {
        double t0 = get_time_sec();

        float t = (float)i * 0.016f;
        float ptr_x = 0.5f + 0.35f * (float)sin(t * 1.5f);
        float ptr_y = 0.5f + 0.35f * (float)cos(t * 1.2f);
        float audio_rms = 0.3f + 0.25f * (float)sin(t * 3.0f);

        glUniform1f(loc_time, t);
        glUniform2f(loc_ptr, ptr_x, ptr_y);
        glUniform1f(loc_audio, audio_rms);

        glDrawArrays(GL_TRIANGLES, 0, 6);
        glFinish(); // Ensure GPU execution completes before measuring

        double t1 = get_time_sec();
        frame_times[i] = (t1 - t0) * 1000.0; // In milliseconds

        if (i == TOTAL_FRAMES / 2) {
            glReadPixels(0, 0, WIDTH, HEIGHT, GL_RGBA, GL_UNSIGNED_BYTE, screenshot_pixels);
            save_ppm("spikes/spike_a_shader/liquid_glass_render.ppm", WIDTH, HEIGHT, screenshot_pixels);
        }
    }

    double total_end = get_time_sec();
    double total_time_sec = total_end - total_start;
    double mean_fps = (double)TOTAL_FRAMES / total_time_sec;

    // Sort frame times for percentiles
    qsort(frame_times, TOTAL_FRAMES, sizeof(double), cmp_double);

    double min_ms = frame_times[0];
    double p50_ms = frame_times[TOTAL_FRAMES / 2];
    double p95_ms = frame_times[(int)(TOTAL_FRAMES * 0.95)];
    double p99_ms = frame_times[(int)(TOTAL_FRAMES * 0.99)];
    double max_ms = frame_times[TOTAL_FRAMES - 1];

    printf("\n=== Liquid Glass Shader Benchmark Results ===\n");
    printf("Total Frames Rendered : %d\n", TOTAL_FRAMES);
    printf("Resolution            : %dx%d (Full HD)\n", WIDTH, HEIGHT);
    printf("Total Duration        : %.3f seconds\n", total_time_sec);
    printf("Mean Framerate        : %.1f FPS\n", mean_fps);
    printf("Median Frame Time     : %.2f ms\n", p50_ms);
    printf("95th Percentile       : %.2f ms\n", p95_ms);
    printf("99th Percentile       : %.2f ms\n", p99_ms);
    printf("Min Frame Time        : %.2f ms\n", min_ms);
    printf("Max Frame Time        : %.2f ms\n", max_ms);
    printf("--------------------------------------------\n");

    int pass = (mean_fps >= 60.0);
    if (pass) {
        printf("VERDICT: [PASS] Shader comfortably exceeds 60 FPS gate on real Linux GPU!\n");
    } else {
        printf("VERDICT: [FAIL] Shader below 60 FPS gate (fallback tiers required).\n");
    }

    // Write JSON report
    FILE* json_f = fopen("spikes/spike_a_shader/benchmark_result.json", "w");
    if (json_f) {
        fprintf(json_f, "{\n");
        fprintf(json_f, "  \"resolution\": \"%dx%d\",\n", WIDTH, HEIGHT);
        fprintf(json_f, "  \"total_frames\": %d,\n", TOTAL_FRAMES);
        fprintf(json_f, "  \"mean_fps\": %.2f,\n", mean_fps);
        fprintf(json_f, "  \"p50_frame_time_ms\": %.3f,\n", p50_ms);
        fprintf(json_f, "  \"p95_frame_time_ms\": %.3f,\n", p95_ms);
        fprintf(json_f, "  \"p99_frame_time_ms\": %.3f,\n", p99_ms);
        fprintf(json_f, "  \"min_frame_time_ms\": %.3f,\n", min_ms);
        fprintf(json_f, "  \"max_frame_time_ms\": %.3f,\n", max_ms);
        fprintf(json_f, "  \"passed_60fps_gate\": %s\n", pass ? "true" : "false");
        fprintf(json_f, "}\n");
        fclose(json_f);
    }

    free(screenshot_pixels);
    glDeleteVertexArrays(1, &vao);
    glDeleteBuffers(1, &vbo);
    glDeleteProgram(program);
    glfwDestroyWindow(window);
    glfwTerminate();

    return pass ? 0 : 2;
}
