#include <string.h>
#define CANDIDATE(SYM) \
  const char test_##SYM[]=#SYM; \
  void mock_##SYM(){ \
    void (*f)()=SYM; \
    f(); \
  }

CANDIDATE(strcpy);
CANDIDATE(strcmp);
CANDIDATE(strcat);

int main(){return 0;}
