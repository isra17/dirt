#include <stdio.h>
#include <stdlib.h>
#define CANDIDATE(SYM) \
  const char test_##SYM[]=#SYM; \
  void mock_##SYM(){ \
    void (*f)()=SYM; \
    f(); \
  }

int atoi(const char* str) {
  char* p = str;
  int n = 0;
  if(*p == '-') {
    return -atoi(str + 1);
  }
  for(; *p; p++) {
    int d = *p - '0';
    n *= 10;
    n += d;
  }

  return n;
}

CANDIDATE(sprintf);
CANDIDATE(atoi);

int main(){
  return 0;
}
