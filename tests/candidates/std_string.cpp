#include <string>

extern "C" {

#define CPP_CANDIDATE(CLASS, NAME, DESC, ...) \
  extern const char test_##NAME##_expect[] = #CLASS "::" #DESC; \
  extern CLASS* test_##NAME##_fn

CPP_CANDIDATE(std::string, std_string_string, string())
(std::string *self) { return new (self)std::string(); }

CPP_CANDIDATE(std::string, std_string_string_char, string(char*))
(std::string *self, char* s) { return new (self)std::string(s); }

CPP_CANDIDATE(std::string, std_string_string_string, string(std::string*))
(std::string *self, std::string* s) { return new (self)std::string(*s); }

CPP_CANDIDATE(std::string, std_string_string_append_char, string(char*))
(std::string *self, char* s) { return new (self)std::string(s); }

}

int main() {
  return 0;
}
