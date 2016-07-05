#include <stdio.h>
#include <stdlib.h>
#define CANDIDATE(SYM) \
  const char test_##SYM[]=#SYM; \
  void mock_##SYM(){ \
    void (*f)()=SYM; \
    f(); \
  }

CANDIDATE(sprintf);

int main(){
  return 0;
}
