#include <GL/gl.h>
#include <GL/glx.h>
#include <GLFW/glfw3.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>

static void error_callback(int error, const char *description) {
  fputs(description, stderr);
}
static void key_callback(GLFWwindow *window, int key, int scancode, int action,
                         int mods) {
  if (key == GLFW_KEY_ESCAPE && action == GLFW_PRESS)
    glfwSetWindowShouldClose(window, GL_TRUE);
}
int main(void) {
  GLFWwindow *window;
  glfwSetErrorCallback(error_callback);
  if (!glfwInit())
    exit(EXIT_FAILURE);

  glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 4);
  glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 6);
  glfwWindowHint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);

  window = glfwCreateWindow(640, 480, "Little test", NULL, NULL);
  if (!window) {
    glfwTerminate();
    exit(EXIT_FAILURE);
  }
  glfwMakeContextCurrent(window);

  glfwSetKeyCallback(window, key_callback);

  printf("OpenGL Version from app: %s\n", glGetString(GL_VERSION));
  printf("OpenGL Vendor from app: %s\n", glGetString(GL_VENDOR));

  glClearColor(0.1, 0.2, 0.3, 1.0);

  double frame = 0.0f;

  while (!glfwWindowShouldClose(window)) {
    glfwSwapBuffers(window);
    glClearColor(cos(frame), sin(frame), cos(-frame), 1.0f);
    glClear(GL_COLOR_BUFFER_BIT);
    glfwPollEvents();

    frame += 0.01;
  }
  glfwDestroyWindow(window);
  glfwTerminate();
  exit(EXIT_SUCCESS);
}