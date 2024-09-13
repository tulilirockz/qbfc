int main() {
  char stack[30000];
  int pointer = 0;
  stack[pointer] = 10;
  while (stack[pointer] != 0) {    
    stack[pointer] -= 1;
    stack[pointer] -= 1;
    stack[pointer] -= 1;
    stack[pointer] -= 1;
    while (stack[pointer] != 0) {    
      stack[pointer] -= 1;
    }
    stack[pointer] -= 1;
  }
    stack[pointer] -= 1;
}
