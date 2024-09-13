#include<stdio.h>

int main() {
  char stack[30000];
  int pointer = 0;
  stack[pointer] += 10;
  while(stack[pointer] != 0) {
    pointer += 1;
    stack[pointer] += 10;
    pointer -= 1;
    stack[pointer] -= 1;
    while(stack[pointer] != 0) {
      pointer -= 1;
    }
  }
  stack[pointer] += 50;
  putchar(stack[pointer]);
}
