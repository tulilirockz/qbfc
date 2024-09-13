#include <stdio.h>

int main() {
  char stack[30000];
  int pointer = 0;
  pointer += 1;
  stack[pointer] += 8;
  while(stack[pointer] != 0) {
    pointer -= 1;
    stack[pointer] += 9;
    pointer += 1;
    stack[pointer] -= 1;
  }
  pointer -= 1;
  putchar(stack[pointer]);
  pointer += 1;
  stack[pointer] += 4;
  while(stack[pointer] != 0) {
    pointer -= 1;
    stack[pointer] += 7;
    pointer += 1;
    stack[pointer] -= 1;
  }
  pointer -= 1;
  stack[pointer] += 1;
  putchar(stack[pointer]);
  stack[pointer] += 7;
  putchar(stack[pointer]);
  putchar(stack[pointer]);
  stack[pointer] += 3;
  putchar(stack[pointer]);
  pointer += 2;
  stack[pointer] += 6;
  while(stack[pointer] != 0) {
    pointer -= 1;
    stack[pointer] += 7;
    pointer += 1;
    stack[pointer] -= 1;
  }
  pointer -= 1;
  stack[pointer] += 2;
  putchar(stack[pointer]);
  stack[pointer] -= 12;
  putchar(stack[pointer]);
  pointer += 1;
  stack[pointer] += 6;
  while (stack[pointer] != 0) {
    pointer -= 1;
    stack[pointer] += 9;
    pointer += 1;
    stack[pointer] -= 1;
  }
  pointer -= 1;
  stack[pointer] += 1;
  putchar(stack[pointer]);
  pointer -= 1;
  putchar(stack[pointer]);
  stack[pointer] += 3;
  putchar(stack[pointer]);
  stack[pointer] -= 6;
  putchar(stack[pointer]);
  stack[pointer] -= 8;
  putchar(stack[pointer]);
  pointer += 3;
  stack[pointer] += 4;
  while (stack[pointer] != 0) {
    pointer -= 1;
    stack[pointer] += 8;
    pointer += 1;
    stack[pointer] -= 1;
  }
  pointer -= 1;
  stack[pointer] += 1;
  putchar(stack[pointer]);
}
